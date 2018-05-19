use website_icon_extract;

pub fn check(homepage: &str, old_favicon: &str) -> String {
    if old_favicon == "" {
        println!("Check for favicon: {}", homepage);
        let icons = website_icon_extract::extract_icons(homepage, "TEST", 5);
        match icons {
            Ok(icons)=>{
                if icons.len() > 0 {
                    println!("Favicon {}", icons[0]);
                    return icons[0].clone();
                }else{
                    println!("No favicons found for: {}", homepage);
                }
            }
            Err(e)=>{
                println!("Favicon error: {}", e.to_string());
            }
        }
    }
    String::from(old_favicon)
}