use reqwest;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::USER_AGENT;
use std::time::Duration;
use website_icon_extract;

pub fn check(
    homepage: &str,
    old_favicon: &str,
    verbosity: u8,
    useragent: &str,
    timeout: u32,
) -> String {
    let check = check_url(old_favicon, useragent, timeout);
    if !check {
        if verbosity > 0 {
            println!("Check for favicon: {}", homepage);
        }
        let icons = website_icon_extract::extract_icons(homepage, useragent, timeout);
        match icons {
            Ok(icons) => {
                if icons.len() > 0 {
                    if verbosity > 0 {
                        println!("Favicon {}", icons[0]);
                    }
                    return icons[0].clone();
                } else {
                    if verbosity > 0 {
                        println!("No favicons found for: {}", homepage);
                    }
                }
            }
            Err(e) => {
                if verbosity > 0 {
                    println!("Favicon error ({}): {}", homepage, e.to_string());
                }
            }
        }
        String::from("")
    } else {
        String::from(old_favicon)
    }
}

fn check_url(url: &str, useragent: &str, timeout: u32) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout.into()))
        .build();
    if client.is_err() {
        return false;
    }
    let client = client.unwrap();
    let res = client
        .get(url)
        .header(USER_AGENT, useragent.to_string())
        .send();
    match res {
        Ok(r) => {
            if r.status().is_success() {
                let t = r.headers().get(CONTENT_TYPE);
                match t {
                    Some(t) => {
                        let value = t.to_str();
                        if let Ok(value) = value {
                            if value.starts_with("image"){
                                return true;
                            }
                        }
                    }
                    None => {
                        return false;
                    }
                }
            }
            return false;
        }
        Err(_) => {
            return false;
        }
    }
}
