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
use bytes::{Buf, BytesMut};
use std::io::Cursor;

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
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            // Allocate the buffer with 1kb of capacity. - 4096
            buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_frame<F, T>(&mut self, mut func: F) -> io::Result<T>
    where
        F: FnMut(&[u8]) -> T,
    {
        loop {
            trace!("read_frame_loop");
            let mut buf = Cursor::new(&self.buffer[..]);

            // Scan the bytes directly
            let start = buf.position() as usize;
            // Scan to the second to last byte
            let end = buf.get_ref().len();

            if end >= start + 2 {
                for i in start..end-1 {
                    if &buf.get_ref()[i..i + 2] == b"\r\n" {
                        // found \r\n, call func to parse frame for cmd or value
                        let result = func(&buf.get_ref()[..i+2]);
                        // We found a line, update the position to be *after* the \n
                        self.buffer.advance(i + 2);
                        // Return the line
                        return Ok(result);
                    }
                }
                // incomplete - maybe error or?
                buf.set_position(end as u64)
            }


            // There is not enough buffered data to read a frame. Attempt to
            // read more data from the socket.
            //
            // On success, the number of bytes is returned. `0` indicates "end
            // of stream".
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                if self.buffer.is_empty() {
                    return Err(Error::from(io::ErrorKind::ConnectionReset));
                } else {
                    return Err(Error::from(io::ErrorKind::ConnectionReset));
                }
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
