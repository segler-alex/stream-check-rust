extern crate native_tls;
extern crate url;
extern crate chrono;

#[macro_use]
extern crate diesel;

use std::env;

pub mod schema;
pub mod models;

mod request;
use request::Request;

mod db;

fn main() {
    let conn = db::establish_connection();
    db::get_stations(conn);

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
