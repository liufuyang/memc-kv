use crate::metrics::METRIC_REQUEST_DURATION_MEMC;
use crate::parser::ascii::parse_ascii_cmd;
use crate::parser::Cmd;
use kv_cache::Cache;
use log::{debug, info, trace};
use nom::AsBytes;
use std::time::SystemTime;
use tokio::io::Error;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct MemcacheServer {
    cache: Cache<Vec<u8>, Vec<u8>>,
}

impl MemcacheServer {
    pub fn new(cache: Cache<Vec<u8>, Vec<u8>>) -> Self {
        MemcacheServer { cache }
    }

    pub async fn serve(&self) -> () {
        // Bind the listener to the address
        let addr = "0.0.0.0:6001";
        info!("Memcache server listening on http://{}, use `nc -c localhost 6001` to connect and test", addr);
        let listener = TcpListener::bind(addr).await.unwrap();

        let memcached = async {
            loop {
                // The second item contains the IP and port of the new connection.
                let (socket, _) = listener.accept().await.unwrap();
                debug!("getting a new connection");
                self.process(socket).await;
            }
        };

        memcached.await
    }

    async fn process(&self, socket: TcpStream) {
        // Write data in the background
        let cache = self.cache.clone();
        tokio::spawn(async move {
            let mut connection = Connection::new(socket);
            loop {
                trace!("process loop");
                let mut start_time = SystemTime::now();
                let cmd_raw = connection
                    .read_frame(|frame| {
                        start_time = SystemTime::now();
                        parse_ascii_cmd(frame)
                            .map(|r| r.1)
                            .map_err(|e| e.to_string())
                    })
                    .await;
                trace!("process loop - got cmd");
                match cmd_raw {
                    Ok(parse_result) => {
                        match parse_result {
                            Ok(cmd) => {
                                trace!("cmd: {:?}", cmd);
                                match cmd {
                                    Cmd::CmdSet {
                                        key,
                                        flag,
                                        ttl,
                                        len,
                                        noreply,
                                    } => {
                                        // reading value
                                        trace!(
                                            "cmd set key: {}",
                                            std::str::from_utf8(key.as_bytes()).unwrap()
                                        );
                                        let value =
                                            connection.read_frame(|frame| frame.to_vec()).await;
                                        match value {
                                            Ok(mut v) => {
                                                trace!("GOT set value: {:?}", v);
                                                if v.len() != (len + 2) as usize {
                                                    if let Err(_e) = connection
                                                        .write_frame(
                                                            "CLIENT_ERROR bad data chunk"
                                                                .as_bytes(),
                                                        )
                                                        .await
                                                    {
                                                        break;
                                                    }
                                                    continue; // value input after set is invalid
                                                }
                                                v.pop();
                                                v.pop();
                                                cache.insert_with_ttl(key, v, ttl, flag);
                                                if !noreply.unwrap_or(false) {
                                                    if let Err(_e) = connection
                                                        .write_frame("STORED".as_bytes())
                                                        .await
                                                    {
                                                        break;
                                                    }
                                                }
                                                let duration = SystemTime::now()
                                                    .duration_since(start_time)
                                                    .unwrap();
                                                METRIC_REQUEST_DURATION_MEMC
                                                    .with_label_values(&["set"])
                                                    .observe(duration.as_secs_f64());
                                            }
                                            // read_frame error when reading a value
                                            Err(e) => {
                                                debug!(
                                                    "read_frame error when reading value: {}",
                                                    e.to_string()
                                                );
                                                break;
                                            }
                                        }
                                    }
                                    Cmd::CmdGet { mut key } => {
                                        trace!(
                                            "cmd set key: {}",
                                            std::str::from_utf8(key.as_bytes()).unwrap()
                                        );
                                        match cache.get(&key) {
                                            Some(value) => {
                                                let mut len = value.len().to_string().into_bytes();
                                                let mut flag =
                                                    value.get_flag().to_string().into_bytes();
                                                let mut value_header =
                                                    Vec::<u8>::with_capacity(6 + key.len() + 100);
                                                value_header.append(&mut b"VALUE ".to_vec());
                                                value_header.append(&mut key);
                                                value_header.append(&mut b" ".to_vec());
                                                value_header.append(&mut flag);
                                                value_header.append(&mut b" ".to_vec());
                                                value_header.append(&mut len);
                                                if let Err(_e) = connection
                                                    .write_frame(value_header.as_bytes())
                                                    .await
                                                    .and(
                                                        connection
                                                            .write_frame(value.as_bytes())
                                                            .await,
                                                    )
                                                {
                                                    break;
                                                }
                                            }
                                            None => (),
                                        };
                                        if let Err(_e) =
                                            connection.write_frame("END".as_bytes()).await
                                        {
                                            break;
                                        }
                                        let duration =
                                            SystemTime::now().duration_since(start_time).unwrap();
                                        METRIC_REQUEST_DURATION_MEMC
                                            .with_label_values(&["get"])
                                            .observe(duration.as_secs_f64());
                                    }
                                    Cmd::CmdVersion => {
                                        if let Err(_e) =
                                            connection.write_frame("VERSION 0.1.0".as_bytes()).await
                                        {
                                            break;
                                        }
                                    }
                                }
                            }
                            // parse error
                            Err(e) => {
                                debug!("parse error: {}", e.to_string());
                                if let Err(write_e) =
                                    // Override error as "ERROR" for cmd parsing error
                                    connection.write_frame(b"ERROR").await
                                {
                                    info!("write_frame error: {}", write_e.to_string());
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("read_frame error when reading cmd: {}", e.to_string());
                        break;
                    }
                }
            }
        });
    }
}

struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    cursor: usize,
    head: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            // Allocate the buffer with 1kb of capacity. - 1024
            buffer: vec![0; 1024],
            cursor: 0,
            head: 0,
        }
    }

    pub async fn read_frame<F, T>(&mut self, mut func: F) -> io::Result<T>
    where
        F: FnMut(&[u8]) -> T,
    {
        loop {
            trace!("read_frame_loop");
            if self.cursor > 0 {
                let start = self.head;
                let end = self.cursor - 1;

                for i in start..end {
                    if &self.buffer[i..i + 2] == b"\r\n" {
                        // found \r\n, call func to parse frame for cmd or value
                        let result = func(&self.buffer[self.head..i + 2]);
                        if i + 1 == end {
                            // no extra behind cursor
                            self.cursor = 0;
                            self.head = 0;
                        } else {
                            // has extra stuff behind i+2
                            self.head = i + 2;
                        }
                        return Ok(result);
                    }
                }
                // incomplete - maybe error or?
            }

            // Ensure the buffer has capacity
            if self.buffer.len() == self.cursor {
                let new_len = self.cursor * 2;
                if new_len > 4096 * 1024 {
                    // 4096 = Max 4kb input size
                    self.cursor = 0;
                    return Err(Error::from(io::ErrorKind::FileTooLarge));
                }
                trace!("read_frame_loop - buffer resize {}", new_len);
                // Grow the buffer
                self.buffer.resize(new_len, 0);
            }

            // Read into the buffer, tracking the number
            // of bytes read
            let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

            if 0 == n {
                if self.cursor == 0 {
                    return Err(Error::from(io::ErrorKind::ConnectionReset));
                } else {
                    // Maybe use a different error for this case?
                    self.cursor = 0;
                    return Err(Error::from(io::ErrorKind::ConnectionReset));
                }
            } else {
                // Update our cursor
                self.cursor += n;
            }
            trace!("read_frame_loop - end");
        }
    }

    async fn write_frame(&mut self, frame: &[u8]) -> io::Result<()> {
        trace!("write_frame - '{}'", std::str::from_utf8(frame).unwrap());
        self.stream.write_all(frame).await?;
        self.stream.write_all(b"\r\n").await?;

        self.stream.flush().await?;

        Ok(())
    }
}
