use native_tls::TlsConnector;

use std::fmt;

use std::net::TcpStream;
use std::io::{Read, Write};

use std::error::Error;
use url::Url;
use std::collections::HashMap;

type BoxResult<T> = Result<T, Box<Error>>;

#[derive(Debug)]
struct RequestError {
    details: String,
}

impl RequestError {
    fn new(msg: &str) -> RequestError {
        RequestError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RequestError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct HttpHeaders {
    pub code: u32,
    pub message: String,
    pub version: String,
    pub headers: HashMap<String, String>,
}

pub struct Request {
    url: Url,
    pub info: HttpHeaders,
    readable: Box<Read>,
    content_read_done: bool,
    content: String,
}

use std::time::Duration;
use std::net::{SocketAddr, ToSocketAddrs};
use std::net::IpAddr;

impl Request {
    pub fn new(url_str: &str) -> BoxResult<Request> {
        let url = Url::parse(url_str)?;

        let host = url.host_str()
            .ok_or(RequestError::new("illegal host name"))?;
        let port = url.port_or_known_default()
            .ok_or(RequestError::new("port unknown"))?;

        let connect_str = format!("{}:{}", host, port);
        let mut addrs_iter = connect_str.to_socket_addrs()?;
        let mut stream = TcpStream::connect_timeout(
            &addrs_iter
                .next()
                .ok_or(RequestError::new("unable to resolve hostname"))?,
            Duration::from_millis(5 * 1000),
        )?;

        if url.scheme() == "https" {
            let connector = TlsConnector::builder()?.build()?;
            let mut stream = connector.connect(host, stream)?;
            Request::send_request(&mut stream, &host, url.path())?;
            let header = Request::read_request(&mut stream)?;
            Ok(Request {
                url: url.clone(),
                info: header,
                readable: Box::new(stream),
                content_read_done: false,
                content: String::from(""),
            })
        } else if url.scheme() == "http" {
            Request::send_request(&mut stream, &host, url.path())?;
            let header = Request::read_request(&mut stream)?;
            Ok(Request {
                url: url.clone(),
                info: header,
                readable: Box::new(stream),
                content_read_done: false,
                content: String::from(""),
            })
        } else {
            Err(Box::new(RequestError::new("unknown scheme")))
        }
    }

    pub fn read_content(&mut self) {
        if self.content_read_done {
            return;
        }
        self.content_read_done = true;

        let content_length: usize = self.info
            .headers
            .get("content-length")
            .unwrap_or(&String::from(""))
            .parse()
            .unwrap_or(10000);

        let mut buffer = vec![0; content_length];
        self.readable.read_exact(&mut buffer);

        let out = String::from_utf8(buffer);
        match out {
            Ok(out) => {
                self.content = out;
            }
            _ => {}
        }
    }

    pub fn get_content<'a>(&'a self) -> &'a str {
        &self.content
    }

    fn read_stream_until(stream: &mut Read, condition: &'static [u8]) -> BoxResult<String> {
        let mut buffer = vec![0; 1];
        let mut bytes = Vec::new();
        loop {
            let result_recv = stream.read(&mut buffer);
            match result_recv {
                Ok(a) => {
                    if a == 0 {
                        break;
                    } else {
                        bytes.push(buffer[0]);
                        if bytes.len() >= condition.len() {
                            let (_, right) = bytes.split_at(bytes.len() - condition.len());
                            if right == condition {
                                break;
                            }
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }
        let out = String::from_utf8_lossy(&bytes);
        Ok(out.to_string())
    }

    fn send_request(stream: &mut Write, host: &str, path: &str) -> BoxResult<()> {
        let request_str = format!(
            "GET {} HTTP/1.0\r\nHost: {}\r\nAccept: */*\r\nConnection: close\r\n\r\n",
            path, host
        );
        stream.write(request_str.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    fn decode_first_line(line: &str) -> BoxResult<HttpHeaders> {
        if line.starts_with("HTTP/") {
            if line.len() < 14 {
                return Err(Box::new(RequestError::new("HTTP status line too short")));
            }
            Ok(HttpHeaders {
                code: line[9..12].parse()?,
                message: String::from(&line[13..]),
                version: String::from(&line[5..8]),
                headers: HashMap::new(),
            })
        } else if line.starts_with("ICY") {
            Ok(HttpHeaders {
                code: line[4..7].parse()?,
                message: String::from(&line[8..]),
                version: String::from(""),
                headers: HashMap::new(),
            })
        } else {
            return Err(Box::new(RequestError::new("HTTP header missing")));
        }
    }

    fn read_request(stream: &mut Read) -> BoxResult<HttpHeaders> {
        let out = Request::read_stream_until(stream, b"\r\n")?;
        let mut httpinfo = Request::decode_first_line(&out)?;

        let out = Request::read_stream_until(stream, b"\r\n\r\n")?;
        let lines = out.lines();

        for line in lines {
            match line.find(':') {
                Some(index) => {
                    let (key, value) = line.split_at(index);
                    httpinfo.headers.insert(
                        String::from(key).to_lowercase(),
                        String::from(value[1..].trim()),
                    );
                }
                _ => {}
            }
        }
        Ok(httpinfo)
    }
}
