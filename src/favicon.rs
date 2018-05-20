use request;
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
    let r = request::Request::new_recursive(url, "TEST", 5);
    match r {
        Ok(r) => {
            if r.get_code() == 200 {
                let t = r.get_header("content-type");
                if t.is_some() {
                    let t = t.unwrap();
                    if t.starts_with("image") {
                        return true;
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
