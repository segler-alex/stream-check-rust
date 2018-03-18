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
}

impl Request {
    pub fn new(url_str: &str) -> BoxResult<Request> {
        Ok(Request {
            url: Url::parse(url_str)?,
        })
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
        let out = String::from_utf8(bytes)?;
        Ok(out)
    }

    fn send_request(&self, stream: &mut Write, host: &str) -> BoxResult<()> {
        let request_str = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nRange: bytes=0-1\r\nConnection: close\r\n\r\n",
            self.url.path(),
            host
        );
        stream.write(request_str.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    pub fn connect(&self) -> BoxResult<HttpHeaders> {
        let host = self.url
            .host_str()
            .ok_or(RequestError::new("illegal host name"))?;
        let port = self.url
            .port_or_known_default()
            .ok_or(RequestError::new("port unknown"))?;
        println!("Connect to {}", self.url);
        println!("Scheme: {}", self.url.scheme());
        println!("Host: {}", host);
        println!("Port: {}", port);
        println!("Path: {}", self.url.path());

        let connect_str = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(connect_str)?;

        if self.url.scheme() == "https" {
            let connector = TlsConnector::builder().unwrap().build()?;
            let mut stream = connector.connect(host, stream)?;
            self.send_request(&mut stream, &host)?;
            Request::read_request(&mut stream)
        } else if self.url.scheme() == "http" {
            self.send_request(&mut stream, &host)?;
            Request::read_request(&mut stream)
        } else {
            Err(Box::new(RequestError::new("unknown scheme")))
        }
    }

    fn read_request(stream: &mut Read) -> BoxResult<HttpHeaders> {
        let out = Request::read_stream_until(stream, b"\r\n")?;

        if !out.starts_with("HTTP/") {
            return Err(Box::new(RequestError::new("not http!")));
        }

        if out.len() < 14 {
            return Err(Box::new(RequestError::new("http status line too short")));
        }

        let mut httpinfo = HttpHeaders {
            code: out[9..12].parse()?,
            message: String::from(&out[13..]),
            version: String::from(&out[5..8]),
            headers: HashMap::new(),
        };

        let out = Request::read_stream_until(stream, b"\r\n\r\n")?;
        let lines = out.lines();

        for line in lines {
            match line.find(':') {
                Some(index) => {
                    let (key, value) = line.split_at(index);
                    httpinfo
                        .headers
                        .insert(String::from(key), String::from(value[1..].trim()));
                }
                _ => {}
            }
        }
        Ok(httpinfo)
    }
}
