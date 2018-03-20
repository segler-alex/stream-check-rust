extern crate chrono;
extern crate native_tls;
extern crate threadpool;
extern crate url;

#[macro_use]
extern crate diesel;

use std::env;

pub mod schema;
pub mod models;

use threadpool::ThreadPool;

mod request;
mod db;
mod streamcheck;
mod pls;

fn debugcheck(url: &str) {
    let items = streamcheck::check(&url);
    for item in items {
        println!("{:?}", item);
    }
}

fn dbcheck() {
    let conn = db::establish_connection();
    let stations = db::get_stations(conn, 50);

    let n_workers = 1;
    let pool = ThreadPool::new(n_workers);
    println!("dbcheck()");
    for station in stations {
        println!("queued {}", station.Name);
        pool.execute(move || {
            println!("started {} - {}", station.Name, station.Url);
            debugcheck(&station.Url);
            println!("finished {}\n", station.Name);
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
