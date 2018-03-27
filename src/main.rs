extern crate chrono;
extern crate native_tls;
extern crate playlist_decoder;
extern crate threadpool;
extern crate url;

extern crate quick_xml;

#[macro_use]
extern crate diesel;

use std::env;

pub mod models;
pub mod schema;

use threadpool::ThreadPool;

mod db;
mod request;
mod streamcheck;

fn debugcheck(url: &str) {
    let items = streamcheck::check(&url);
    for item in items {
        println!("{:?}", item);
    }
}

fn dbcheck(concurrency: usize) {
    let conn = db::establish_connection();
    let stations = db::get_stations(conn, 50);

    let pool = ThreadPool::new(concurrency);
    for station in stations {
        pool.execute(move || {
            debugcheck(&station.Url);
        });
    }
    pool.join();
}

fn main() {
    let concurrency: usize = env::var("CONCURRENCY")
        .unwrap_or(String::from("10"))
        .parse()
        .expect("threads is not number");

    match env::args().nth(1) {
        Some(url) => {
            debugcheck(&url);
        }
        None => {
            dbcheck(concurrency);
        }
    };
}
