extern crate native_tls;
extern crate url;
extern crate chrono;

#[macro_use]
extern crate diesel;

use std::env;

pub mod schema;
pub mod models;

mod request;
mod db;
mod streamcheck;
mod pls;

fn main() {
    /*let conn = db::establish_connection();
    db::get_stations(conn);*/

    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return;
        }
    };


    let items = streamcheck::check(&url);
    for item in items{
        println!("{:?}", item);
    }
}
