extern crate native_tls;
extern crate playlist_decoder;
extern crate threadpool;
extern crate url;
extern crate hostname;
extern crate mysql;
extern crate quick_xml;

use std::env;

pub mod models;

use threadpool::ThreadPool;

mod db;
mod request;
mod streamcheck;

use std::time::Duration;
use hostname::get_hostname;
use std::thread;

use models::StationCheckItemNew;

fn debugcheck(url: &str) {
    let items = streamcheck::check(&url);
    for item in items {
        println!("{:?}", item);
    }
}

fn dbcheck(connection_str: &str, source: &str, concurrency: usize, stations_count: u32) {
    let conn = db::new(connection_str);
    if let Ok(conn) = conn {
        let stations = db::get_stations_to_check(&conn, 24, stations_count);

        let pool = ThreadPool::new(concurrency);
        for station in stations {
            let source = String::from(source);
            let conn = conn.clone();
            pool.execute(move || {
                let items = streamcheck::check(&station.url);
                let mut working = false;
                for item in items.iter() {
                    match item {
                        &Ok(ref item) => {
                            let new_item = StationCheckItemNew {
                                station_uuid: station.uuid.clone(),
                                source: source.clone(),
                                codec: item.Codec.clone(),
                                bitrate: item.Bitrate as i32,
                                hls: item.Hls,
                                check_ok: true,
                                url: item.Url.clone(),
                            };
                            db::insert_check(&conn, &new_item);
                            db::update_station(&conn, &new_item);
                            working = true;
                            println!("OK {} - {:?}", station.uuid, item);
                            break;
                        }
                        &Err(_) => {}
                    }
                }

                if !working {
                    let new_item = StationCheckItemNew {
                        station_uuid: station.uuid.clone(),
                        source: source.clone(),
                        codec: "".to_string(),
                        bitrate: 0,
                        hls: false,
                        check_ok: false,
                        url: "".to_string(),
                    };
                    db::insert_check(&conn, &new_item);
                    db::update_station(&conn, &new_item);
                    println!("FAIL {}", station.uuid);
                }
            });
        }
        pool.join();
    }
}

fn main() {
    let concurrency: usize = env::var("CONCURRENCY")
        .unwrap_or(String::from("10"))
        .parse()
        .expect("CONCURRENCY is not number");
    let check_stations: u32 = env::var("STATIONS")
        .unwrap_or(String::from("50"))
        .parse()
        .expect("CONCURRENCY is not number");
    let do_loop: bool = env::var("LOOP")
        .unwrap_or(String::from("false"))
        .parse()
        .expect("LOOP is not bool");
    let pause_seconds: u64 = env::var("PAUSE_SECONDS")
        .unwrap_or(String::from("10"))
        .parse()
        .expect("PAUSE_SECONDS is not u64");
    let source: String = env::var("SOURCE")
        .unwrap_or(String::from(get_hostname().unwrap_or("".to_string())));
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    println!("DATABASE_URL  : {}", database_url);
    println!("LOOP          : {}", do_loop);
    println!("SOURCE        : {}", source);
    println!("CONCURRENCY   : {}", concurrency);
    println!("STATIONS      : {}", check_stations);
    println!("PAUSE_SECONDS : {}", pause_seconds);

    loop {
        match env::args().nth(1) {
            Some(url) => {
                debugcheck(&url);
            }
            None => {
                dbcheck(&database_url, &source, concurrency, check_stations);
            }
        };
        if !do_loop{
            break;
        }

        println!("Waiting.. ({} secs)", pause_seconds);
        thread::sleep(Duration::from_secs(pause_seconds));
    }
}
