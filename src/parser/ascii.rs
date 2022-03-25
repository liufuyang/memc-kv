use crate::parser::Cmd;
use btoi::btou;
use log::debug;
use nom::bytes::streaming::tag_no_case;
use nom::error::{Error, ErrorKind};
use nom::{
    branch::alt,
    bytes::streaming::{tag, take_while1, take_while_m_n},
    character::{is_digit, streaming::crlf},
    combinator::{map_res, opt, value},
    sequence::tuple,
    IResult,
};

fn is_key_char(chr: u8) -> bool {
    chr > 32 && chr < 127
}

fn parse_ascii_u32(buf: &[u8]) -> IResult<&[u8], u32> {
    map_res(take_while_m_n(1, 10, is_digit), btou)(buf)
}

pub(crate) fn parse_ascii_cmd(buf: &[u8]) -> IResult<&[u8], Cmd> {
    // debug!("Parsing: '{}'", std::str::from_utf8(buf).unwrap());
    if buf.is_empty() {
        debug!("Parsing with empty buf");
        return Err(nom::Err::Error(Error::new(buf, ErrorKind::Eof)));
    }

    let (buf, c) = alt((
        value("set", tag_no_case(b"set")),
        value("get", tag_no_case(b"get")),
        value("version", tag_no_case(b"version")),
    ))(buf)?;

    match c {
        "set" => {
            let (buf, (_, key, _, flag, _, ttl, _, len, _, noreply, _)) = tuple((
                // VALUE key flags data_len [cas id]\r\n
                // data block\r\n
                tag(" "),
                take_while1(is_key_char),
                tag(" "),
                parse_ascii_u32, // flag
                tag(" "),
                parse_ascii_u32, // ttl
                tag(" "),
                parse_ascii_u32, // len
                opt(tag(" ")),
                opt(alt((value(true, tag_no_case(b"noreply")),))),
                crlf,
            ))(buf)?;
            Ok((
                buf,
                Cmd::CmdSet {
                    key: key.to_vec(),
                    flag,
                    ttl,
                    len,
                    noreply,
                },
            ))
        }
        "get" => {
            // get cmd
            let (buf, (_, key, _)) = tuple((tag(" "), take_while1(is_key_char), crlf))(buf)?;
            Ok((buf, Cmd::CmdGet { key: key.to_vec() }))
        }
        "version" => {
            let (buf, _) = crlf(buf)?;
            Ok((buf, Cmd::CmdVersion))
        }
        _ => {
            panic!("not possible")
        }
    }
}
