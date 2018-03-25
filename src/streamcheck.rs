#![allow(non_snake_case)]
use request::Request;

use std::fmt;
use playlist_decoder;
use url::Url;

#[derive(Debug)]
pub struct StreamInfo{
    pub Name: String,
    pub Description: String,
    pub Type: String,
    pub Url: String,
    pub Homepage: String,
    pub Genre: String,
    pub Bitrate: u32,
    pub Sampling: u32,
    pub Codec: String,
}

#[derive(Debug)]
struct StreamCheckError {
    url: String,
    details: String,
}

impl StreamCheckError {
    fn new(url: &str, msg: &str) -> StreamCheckError {
        StreamCheckError {
            url: url.to_string(),
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for StreamCheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for StreamCheckError {
    fn description(&self) -> &str {
        &self.details
    }
}

use std::error::Error;
type BoxResult<T> = Result<T, Box<Error>>;

fn type_is_m3u(content_type: &str) -> bool {
    return content_type == "application/mpegurl" || content_type == "application/x-mpegurl"
        || content_type == "audio/mpegurl" || content_type == "audio/x-mpegurl"
        || content_type == "application/vnd.apple.mpegurl"
        || content_type == "application/vnd.apple.mpegurl.audio";
}

fn type_is_pls(content_type: &str) -> bool {
    return content_type == "audio/x-scpls" || content_type == "application/x-scpls"
        || content_type == "application/pls+xml";
}

fn type_is_asx(content_type: &str) -> bool {
    return content_type == "video/x-ms-asx" || content_type == "video/x-ms-asf";
}

fn type_is_xspf(content_type: &str) -> bool {
    return content_type == "application/xspf+xml" || content_type == "application/xml";
}

fn type_is_playlist(content_type: &str) -> bool {
    return type_is_m3u(content_type) || type_is_pls(content_type) || type_is_asx(content_type) || type_is_xspf(content_type);
}

fn type_is_stream(content_type: &str) -> Option<&str> {
    match content_type {
        "audio/mpeg" => Some("MP3"),
        "audio/mp3" => Some("MP3"),
        "audio/aac" => Some("AAC"),
        "audio/x-aac" => Some("AAC"),
        "audio/aacp" => Some("AAC+"),
        "audio/ogg" => Some("OGG"),
        "application/ogg" => Some("OGG"),
        "audio/flac" => Some("FLAC"),
        "application/flv" => Some("FLV"),
        "application/octet-stream" => Some("UNKNOWN"),
        _ => None
    }
}

pub fn check(url: &str) -> Vec<BoxResult<StreamInfo>> {
    let request = Request::new(&url);
    let mut list: Vec<BoxResult<StreamInfo>> = vec![];
    match request {
        Ok(mut request) => {
            if request.info.code >= 200 && request.info.code < 300 {
                let mut is_playlist = false;
                let mut is_stream = false;
                let mut stream_type = String::from("");
                {
                    let content_type = request.info.headers.get("content-type");
                    match content_type {
                        Some(content_type) => {
                            if type_is_playlist(content_type) {
                                is_playlist = true;
                            } else if type_is_stream(content_type).is_some() {
                                stream_type = String::from(type_is_stream(content_type).unwrap_or(""));
                                is_stream = true;
                            }else{
                                list.push(Err(Box::new(StreamCheckError::new(url, &format!("unknown content type {}", content_type)))));
                            }
                        }
                        None => {
                            list.push(Err(Box::new(StreamCheckError::new(url, "Missing content-type in http header"))));
                        }
                    }
                }
                if is_playlist {
                    request.read_content();
                    let playlist = decode_playlist(url, request.get_content());
                    if playlist.len() == 0{
                        list.push(Err(Box::new(StreamCheckError::new(url, "Empty playlist"))));
                    } else {
                        list.extend(playlist);
                    }
                }else if is_stream {
                    let stream = StreamInfo{
                        Url: String::from(url),
                        Type: request.info.headers.get("content-type").unwrap_or(&String::from("")).clone(),
                        Name: request.info.headers.get("icy-name").unwrap_or(&String::from("")).clone(),
                        Description: request.info.headers.get("icy-description").unwrap_or(&String::from("")).clone(),
                        Homepage: request.info.headers.get("icy-url").unwrap_or(&String::from("")).clone(),
                        Bitrate: request.info.headers.get("icy-br").unwrap_or(&String::from("")).parse().unwrap_or(0),
                        Genre: request.info.headers.get("icy-genre").unwrap_or(&String::from("")).clone(),
                        Sampling: request.info.headers.get("icy-sr").unwrap_or(&String::from("")).parse().unwrap_or(0),
                        Codec: stream_type,
                    };
                    list.push(Ok(stream));
                }
            } else if request.info.code >= 300 && request.info.code < 400 {
                let location = request.info.headers.get("location");
                match location {
                    Some(location) => {
                        list.extend(check(location));
                    }
                    None => {}
                }
            } else {
                list.push(Err(Box::new(StreamCheckError::new(url, &format!("illegal http status code {}", request.info.code)))));
            }
        }
        Err(err) => list.push(Err(Box::new(StreamCheckError::new(url,&err.to_string())))),
    }
    list
}

fn decode_playlist(url_str:&str, content: &str) -> Vec<BoxResult<StreamInfo>> {
    let mut list = vec![];
    let base_url = Url::parse(url_str);
    match base_url{
        Ok(base_url) => {
            let urls = playlist_decoder::decode(content);
            for url in urls{
                let abs_url = base_url.join(&url);
                match abs_url{
                    Ok(abs_url)=>{
                        println!("base='{}' + src='{}' = '{}'", base_url, url, abs_url);
                        list.extend(check(&abs_url.as_str()));
                    }
                    Err(err)=>{
                        list.push(Err(Box::new(StreamCheckError::new(url_str,&err.to_string()))));
                    }
                }
            }
        }
        Err(err) => {
            list.push(Err(Box::new(StreamCheckError::new(url_str,&err.to_string()))));
        }
    }
    
    list
}