use website_icon_extract;
use request;

pub fn check(homepage: &str, old_favicon: &str, verbosity: u8) -> String {
    let check = check_url(old_favicon, 5);
    if !check {
        if verbosity > 0{
            println!("Check for favicon: {}", homepage);
        }
        let icons = website_icon_extract::extract_icons(homepage, "TEST", 5);
        match icons {
            Ok(icons)=>{
                if icons.len() > 0 {
                    if verbosity > 0{
                        println!("Favicon {}", icons[0]);
                    }
                    return icons[0].clone();
                }else{
                    if verbosity > 0{
                        println!("No favicons found for: {}", homepage);
                    }
                }
            }
            Err(e)=>{
                if verbosity > 0{
                    println!("Favicon error ({}): {}", homepage, e.to_string());
                }
            }
        }
    }
    String::from(old_favicon)
}

fn check_url(url: &str, depth: u8) -> bool {
    let r = request::Request::new_recursive(url, "TEST", 5);
    match r {
        Ok(r)=>{
            if r.get_code() == 200{
                let t = r.get_header("content-type");
                if t.is_some(){
                    let t = t.unwrap();
                    if t.starts_with("image") {
                        return true;
                    }
                }
            }
            if r.get_code() == 301 || r.get_code() == 302 {
                let l = r.get_header("location");
                if l.is_some(){
                    let l = l.unwrap();
                    if depth > 0{
                        return check_url(&l, depth - 1);
                    }
                }
            }
            return false;
        }
        Err(e)=>{
            return false;
        }
    }
}