use reqwest;
use reqwest::header::ContentType;
use reqwest::header::UserAgent;
use std::time::Duration;
use website_icon_extract;

pub fn check(homepage: &str, old_favicon: &str, verbosity: u8) -> String {
    let check = check_url(old_favicon);
    if !check {
        if verbosity > 0 {
            println!("Check for favicon: {}", homepage);
        }
        let icons = website_icon_extract::extract_icons(homepage, "TEST", 5);
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

fn check_url(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build();
    if client.is_err() {
        return false;
    }
    let client = client.unwrap();
    let res = client.get(url).header(UserAgent::new("TEST")).send();
    match res {
        Ok(r) => {
            if r.status().is_success() {
                let t = r.headers().get::<ContentType>();
                match t {
                    Some(t) => {
                        if t.to_string().starts_with("image") {
                            return true;
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
