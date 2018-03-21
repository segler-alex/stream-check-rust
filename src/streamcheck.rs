#![allow(non_snake_case)]
use request::Request;
use pls;
use m3u;
use asx;
use xspf;

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

pub fn check(url: &str) -> Vec<StreamInfo> {
    let request = Request::new(&url);
    let mut list = vec![];
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
                                println!("unknown content type {}", content_type);
                            }
                        }
                        None => {
                            println!("missing content-type in http header");
                        }
                    }
                }
                if is_playlist {
                    request.read_content();
                    let playlist = decode_playlist(request.get_content());
                    if playlist.len() == 0{
                        println!("Empty playlist ({})", url);
                    }
                    list.extend(playlist);
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
            } else {
                println!("illegal http status code {}", request.info.code);
            }
        }
        Err(err) => println!("Connection error: {} - {}", url, err),
    }
    list
}

fn decode_playlist(content: &str) -> Vec<StreamInfo> {
    let mut list = vec![];

    match content.to_lowercase().find("<playlist"){
        Some(_)=>{
            let xspf_items = xspf::decode_playlist(content);
            for item in xspf_items {
                list.extend(check(&item.url));
                list.extend(check(&item.identifier));
            }
        }
        None =>{
            match content.to_lowercase().find("<asx"){
                Some(_)=>{
                    let pls_items = asx::decode_playlist(content);
                    for item in pls_items {
                        list.extend(check(&item.url));
                    }
                }
                None =>{
                    match content.to_lowercase().find("[playlist]"){
                        Some(_) => {
                            let pls_items = pls::decode_playlist(content);
                            for item in pls_items {
                                list.extend(check(&item.url));
                            }
                        }
                        None => {
                            let m3u_items = m3u::decode_playlist(content);
                            for item in m3u_items {
                                list.extend(check(&item.url));
                            }
                        }
                    }
                }
            }
        }
    }
    
    list
}

/*fn getRemoteDirUrl(url: &str) {
    $parsed_url = parse_url($url);
    if ($parsed_url) {
        $scheme = isset($parsed_url['scheme']) ? $parsed_url['scheme'].'://' : '';
        $host = isset($parsed_url['host']) ? $parsed_url['host'] : '';
        $port = isset($parsed_url['port']) ? ':'.$parsed_url['port'] : '';
        $path = isset($parsed_url['path']) ? dirname($parsed_url['path']) : '';
        return "$scheme$host$port$path";
    }

    return null;
}

fn fixPlaylistItem(url: &str, playlistItem: &str) -> BoxResult<String> {
    if (hasCorrectScheme(playlistItem)){
        let remoteDir = getRemoteDirUrl(url);
        if (remoteDir != false){
            Ok(Box::new(format!("{}/{}",remoteDir,playlistItem)))
        }
        Err()
    }
    Ok(playlistItem)
}*/

/*fn hasCorrectScheme(url: &str) -> bool {
    let lower = url.to_lowercase();
    return lower.starts_with("http://") || lower.starts_with("https://");
}*/
