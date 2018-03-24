extern crate chrono;
extern crate native_tls;
extern crate threadpool;
extern crate url;
extern crate playlist_decoder;

extern crate quick_xml;

#[macro_use]
extern crate diesel;

use std::env;

pub mod schema;
pub mod models;

use threadpool::ThreadPool;

mod request;
mod db;
mod streamcheck;

fn debugcheck(url: &str) {
    let items = streamcheck::check(&url);
    for item in items {
        println!("{:?}", item);
    }
}

fn dbcheck() {
    let conn = db::establish_connection();
    let stations = db::get_stations(conn, 50);

    let n_workers = 10;
    let pool = ThreadPool::new(n_workers);
    for station in stations {
        pool.execute(move || {
            debugcheck(&station.Url);
        });
    }
    pool.join();
}

fn main() {
    match env::args().nth(1) {
        Some(url) => {
            debugcheck(&url);
        }
        None => {
            dbcheck();
        }
    };
}
