extern crate chrono;
extern crate native_tls;
extern crate playlist_decoder;
extern crate threadpool;
extern crate url;
extern crate uuid;

extern crate quick_xml;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate diesel;

use std::env;

pub mod models;
pub mod schema;

use threadpool::ThreadPool;

mod db;
mod request;
mod streamcheck;

use diesel::prelude::*;
use self::models::NewStationCheckItem;
use uuid::Uuid;

fn debugcheck(url: &str) {
    let items = streamcheck::check(&url);
    for item in items {
        println!("{:?}", item);
    }
}

fn dbcheck(concurrency: usize) {
    let conn = db::establish_connection();
    let stations = db::get_stations(&conn, 50);

    let pool = ThreadPool::new(concurrency);
    for station in stations {
        pool.execute(move || {
            let items = streamcheck::check(&station.Url);
            let mut working = false;
            for item in items.iter() {
                println!("{:?}", item);

                match item{
                    &Ok(ref item)=>{
                        let my_uuid = Uuid::new_v4();
                        let new_post = NewStationCheckItem {
                            StationUuid: &station.StationUuid,
                            CheckUuid: &my_uuid.to_string(),
                            Source: "",
                            Codec: &item.Codec.clone(),
                            Bitrate: item.Bitrate as i32,
                            Hls: false,
                            CheckOK: true,
                        };
                        let conn = db::establish_connection();
                        diesel::insert_into(schema::StationCheck::table)
                            .values(&new_post)
                            .execute(&conn)
                            .expect("Error saving new post");
                        working = true;
                        break;
                    }
                    &Err(_)=>{

                    }
                }
            }

            if !working{
                 let my_uuid = Uuid::new_v4();
                let new_post = NewStationCheckItem {
                    StationUuid: &station.StationUuid,
                    CheckUuid: &my_uuid.to_string(),
                    Source: "",
                    Codec: "",
                    Bitrate: 0,
                    Hls: false,
                    CheckOK: false,
                };
                let conn = db::establish_connection();
                diesel::insert_into(schema::StationCheck::table)
                    .values(&new_post)
                    .execute(&conn)
                    .expect("Error saving new post");
            }
        });
    }
    pool.join();
}

fn main() {
    let concurrency: usize = env::var("CONCURRENCY")
        .unwrap_or(String::from("10"))
        .parse()
        .expect("CONCURRENCY is not number");

    match env::args().nth(1) {
        Some(url) => {
            debugcheck(&url);
        }
        None => {
            dbcheck(concurrency);
        }
    };
}
