#![allow(non_snake_case)]
use request::Request;
use pls;

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
}

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

fn type_is_playlist(content_type: &str) -> bool {
    return type_is_m3u(content_type) || type_is_pls(content_type);
}

fn type_is_stream(content_type: &str) -> bool {
    return content_type == "audio/mpeg" || content_type == "audio/aacp";
}

pub fn check(url: &str) -> Vec<StreamInfo> {
    let request = Request::new(&url);
    let mut list = vec![];
    match request {
        Ok(mut request) => {
            if request.info.code >= 200 && request.info.code < 300 {
                let mut is_playlist = false;
                let mut is_stream = false;
                {
                    let content_type = request.info.headers.get("content-type");
                    match content_type {
                        Some(content_type) => {
                            if type_is_playlist(content_type) {
                                is_playlist = true;
                            } else if type_is_stream(content_type) {
                                is_stream = true;
                            }
                        }
                        None => {}
                    }
                }
                if is_playlist {
                    request.read_content();
                    list.extend(decode_playlist(request.get_content()));
                }
                if is_stream {
                    let stream = StreamInfo{
                        Url: String::from(url),
                        Type: request.info.headers.get("content-type").unwrap_or(&String::from("")).clone(),
                        Name: request.info.headers.get("icy-name").unwrap_or(&String::from("")).clone(),
                        Description: request.info.headers.get("icy-description").unwrap_or(&String::from("")).clone(),
                        Homepage: request.info.headers.get("icy-url").unwrap_or(&String::from("")).clone(),
                        Bitrate: request.info.headers.get("icy-br").unwrap_or(&String::from("")).parse().unwrap_or(0),
                        Genre: request.info.headers.get("icy-genre").unwrap_or(&String::from("")).clone(),
                        Sampling: request.info.headers.get("icy-sr").unwrap_or(&String::from("")).parse().unwrap_or(0),
                    };
                    list.push(stream);
                }
            } else if request.info.code >= 300 && request.info.code < 400 {
                let location = request.info.headers.get("location");
                match location {
                    Some(location) => {
                        list.extend(check(location));
                    }
                    None => {}
                }
            }
        }
        Err(err) => println!("Connection error: {}", err),
    }
    list
}

fn decode_playlist(content: &str) -> Vec<StreamInfo> {
    let pls_items = pls::decode_playlist_pls(content);
    let mut list = vec![];
    for item in pls_items {
        list.extend(check(&item.url));
    }
    list
}