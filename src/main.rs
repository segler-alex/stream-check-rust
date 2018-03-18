extern crate url;
extern crate native_tls;

use std::env;

mod request;
use request::Request;

fn main() {
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return;
        }
    };

    let request = Request::new(&url).expect("could not parse url");
    let r_connect = request.connect();
    match r_connect {
        Ok(info) => {
            println!("HTTP:   {}", info.version);
            println!("STATUS: {}", info.code);
            println!("STATUS: {}", info.message);
            for header in info.headers {
                println!("{:?}", header);
            }
        }
        Err(err) => println!("{}", err),
    }
}
