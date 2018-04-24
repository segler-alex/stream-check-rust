extern crate hostname;
extern crate mysql;
extern crate native_tls;
extern crate playlist_decoder;
extern crate quick_xml;
extern crate threadpool;
extern crate url;

use std::env;

pub mod models;

use threadpool::ThreadPool;

mod db;
mod request;
mod streamcheck;

use std::time::Duration;
use hostname::get_hostname;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::sync::mpsc::TryRecvError;

use models::StationCheckItemNew;

extern crate colored;

use colored::*;

fn debugcheck(url: &str, timeout: u64) {
    let items = streamcheck::check(&url, true, timeout);
    for item in items {
        println!("{:?}", item);
    }
}

fn check_for_change(old: &models::StationItem, new: &StationCheckItemNew) -> (bool, String) {
    let mut retval = false;
    let mut result = String::from("");

    if old.check_ok != new.check_ok {
        if new.check_ok {
            result.push('+');
            result.red();
        } else {
            result.push('-');
        }
        retval = true;
    } else {
        result.push('~');
    }
    result.push(' ');
    result.push('\'');
    result.push_str(&old.name);
    result.push('\'');
    result.push(' ');
    result.push_str(&old.url);
    if old.hls != new.hls {
        result.push_str(&format!(" hls:{}->{}", old.hls, new.hls));
        retval = true;
    }
    if old.bitrate != new.bitrate {
        result.push_str(&format!(" bitrate:{}->{}", old.bitrate, new.bitrate));
        retval = true;
    }
    if old.codec != new.codec {
        result.push_str(&format!(" codec:{}->{}", old.codec, new.codec));
        retval = true;
    }
    /*if old.urlcache != new.url{
        println!("  url      :{}->{}",old.urlcache,new.url);
        retval = true;
    }*/
    if old.check_ok != new.check_ok {
        if new.check_ok {
            return (retval, result.green().to_string());
        } else {
            return (retval, result.red().to_string());
        }
    } else {
        return (retval, result.yellow().to_string());
    }
}

fn update_station(conn: &mysql::Pool, old: &models::StationItem, new_item: &StationCheckItemNew) {
    db::insert_check(&conn, &new_item);
    db::update_station(&conn, &new_item);
    let (changed, change_str) = check_for_change(&old, &new_item);
    if changed {
        println!("{}", change_str.red());
    }
}

fn dbcheck(connection_str: &str, source: &str, concurrency: usize, stations_count: u32, timeout: u64) {
    let conn = db::new(connection_str);
    if let Ok(conn) = conn {
        let stations = db::get_stations_to_check(&conn, 24, stations_count);

        let pool = ThreadPool::new(concurrency);
        for station in stations {
            let source = String::from(source);
            let conn = conn.clone();
            pool.execute(move || {
                let (_, receiver): (Sender<i32>, Receiver<i32>)= channel();
                let station_name = station.name.clone();
                thread::spawn(move || {
                    for _ in 0..120 {
                        thread::sleep(Duration::from_secs(1));
                        let o = receiver.try_recv();
                        match o {
                            Ok(_) => {
                                return;
                            }
                            Err(value) => match value {
                                TryRecvError::Empty => {
                                    
                                }
                                TryRecvError::Disconnected => {
                                    return;
                                }
                            },
                        }
                    }
                    println!("Still not finished: {}", station_name);
                    std::process::exit(0x0100);
                });
                let mut new_item: StationCheckItemNew = StationCheckItemNew {
                    station_uuid: station.uuid.clone(),
                    source: source.clone(),
                    codec: "".to_string(),
                    bitrate: 0,
                    hls: false,
                    check_ok: false,
                    url: "".to_string(),
                };
                let mut working = false;
                for _i in 1..6 {
                    /*if i > 1{
                        println!("TRY {} - {}", i, station.url);
                    }*/
                    let items = streamcheck::check(&station.url, false, timeout);
                    for item in items.iter() {
                        match item {
                            &Ok(ref item) => {
                                new_item = StationCheckItemNew {
                                    station_uuid: station.uuid.clone(),
                                    source: source.clone(),
                                    codec: item.Codec.clone(),
                                    bitrate: item.Bitrate as i32,
                                    hls: item.Hls,
                                    check_ok: true,
                                    url: item.Url.clone(),
                                };
                                working = true;
                                break;
                            }
                            &Err(_) => {}
                        }
                    }

                    if working {
                        break;
                    }

                    thread::sleep(Duration::from_secs(10));
                }

                update_station(&conn, &station, &new_item);
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
    let tcp_timeout: u64 = env::var("TCP_TIMEOUT")
        .unwrap_or(String::from("10"))
        .parse()
        .expect("TCP_TIMEOUT is not u64");
    let source: String =
        env::var("SOURCE").unwrap_or(String::from(get_hostname().unwrap_or("".to_string())));
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    println!("DATABASE_URL  : {}", database_url);
    println!("LOOP          : {}", do_loop);
    println!("SOURCE        : {}", source);
    println!("CONCURRENCY   : {}", concurrency);
    println!("STATIONS      : {}", check_stations);
    println!("PAUSE_SECONDS : {}", pause_seconds);
    println!("TCP_TIMEOUT   : {}", tcp_timeout);

    let conn = db::new(&database_url);
    if let Ok(conn) = conn {
        loop {
            match env::args().nth(1) {
                Some(url) => {
                    debugcheck(&url, tcp_timeout);
                }
                None => {
                    dbcheck(&database_url, &source, concurrency, check_stations, tcp_timeout);
                }
            };
            if !do_loop {
                break;
            }

            let checks_hour = db::get_checks(&conn,1,&source);
            let checks_day = db::get_checks(&conn,24,&source);
            let stations_broken = db::get_station_count_broken(&conn);
            let stations_working = db::get_station_count_working(&conn);
            println!("Waiting.. ({} secs) {} Checks/Hour, {} Checks/Day, {} Working stations, {} Broken stations", pause_seconds, checks_hour, checks_day, stations_working, stations_broken);
            thread::sleep(Duration::from_secs(pause_seconds));
        }
    }
}
