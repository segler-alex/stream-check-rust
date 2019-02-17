extern crate av_stream_info_rust;
#[macro_use]
extern crate clap;
extern crate colored;
extern crate hostname;
#[macro_use]
extern crate mysql;
extern crate native_tls;
extern crate reqwest;
extern crate threadpool;
extern crate url;
extern crate website_icon_extract;

use clap::{App, Arg};

pub mod models;

mod check;
mod db;
mod favicon;

use hostname::get_hostname;
use std::thread;
use std::time::Duration;

fn main() {
    let hostname: String = get_hostname().unwrap_or("".to_string());
    let matches = App::new("stream-check")
        .version(crate_version!())
        .author("segler_alex@web.de")
        .about("Stream check tool for radiobrowser")
/*        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .default_value("/etc/radiobrowser.conf")
                .help("Sets a custom config file")
                .takes_value(true),
        )*/
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .value_name("DATABASE_URL")
                .help("Database connection url")
                .env("DATABASE_URL")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .value_name("SOURCE")
                .help("Source string for database check entries")
                .env("SOURCE")
                .default_value(&hostname)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("useragent")
                .short("u")
                .long("useragent")
                .value_name("USERAGENT")
                .help("user agent value for http requests")
                .env("USERAGENT")
                .default_value("stream-check/0.1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("retries")
                .short("r")
                .long("retries")
                .value_name("RETRIES")
                .help("Max number of retries for station checks")
                .env("RETRIES")
                .default_value("5")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("max_depth")
                .short("m")
                .long("max_depth")
                .value_name("MAX_DEPTH")
                .help("max recursive link check depth")
                .env("MAX_DEPTH")
                .default_value("5")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("tcp_timeout")
                .short("t")
                .long("tcp_timeout")
                .value_name("TCP_TIMEOUT")
                .help("tcp connect/read timeout in seconds")
                .env("TCP_TIMEOUT")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pause_seconds")
                .short("a")
                .long("pause_seconds")
                .value_name("PAUSE_SECONDS")
                .help("database check pauses in seconds")
                .env("PAUSE_SECONDS")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("stations")
                .short("n")
                .long("stations")
                .value_name("STATIONS")
                .help("batch size for station checks")
                .env("STATIONS")
                .default_value("50")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("concurrency")
                .short("c")
                .long("concurrency")
                .value_name("CONCURRENCY")
                .help("streams checked in parallel")
                .env("CONCURRENCY")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("delete")
                .short("x")
                .long("delete")
                .value_name("DELETE")
                .help("delete broken stations according to rules")
                .env("DELETE")
                .default_value("false")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("loop")
                .short("l")
                .long("loop")
                .value_name("LOOP")
                .help("do loop checks forever")
                .env("LOOP")
                .default_value("false")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("favicon")
                .short("f")
                .long("favicon")
                .value_name("FAVICON")
                .help("check favicons and try to repair them")
                .env("FAVICON")
                .default_value("false")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    /*if let Some(config) = matches.value_of("config"){
        println!("found config at {}", config);
    }*/

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    let verbosity: u8 = matches.occurrences_of("v") as u8;
    let concurrency: usize = matches
        .value_of("concurrency")
        .unwrap()
        .parse()
        .expect("concurrency is not usize");
    let check_stations: u32 = matches
        .value_of("stations")
        .unwrap()
        .parse()
        .expect("stations is not u32");
    let do_loop: bool = matches
        .value_of("loop")
        .unwrap()
        .parse()
        .expect("loop is not bool");
    let delete: bool = matches
        .value_of("delete")
        .unwrap()
        .parse()
        .expect("delete is not bool");
    let favicon: bool = matches
        .value_of("favicon")
        .unwrap()
        .parse()
        .expect("favicon is not bool");
    let pause_seconds: u64 = matches
        .value_of("tcp_timeout")
        .unwrap()
        .parse()
        .expect("pause_seconds is not u64");
    let tcp_timeout: u32 = matches
        .value_of("tcp_timeout")
        .unwrap()
        .parse()
        .expect("tcp_timeout is not u32");
    let max_depth: u8 = matches
        .value_of("max_depth")
        .unwrap()
        .parse()
        .expect("max_depth is not u8");
    let retries: u8 = matches
        .value_of("retries")
        .unwrap()
        .parse()
        .expect("retries is not u8");
    let source: String = String::from(matches.value_of("source").unwrap());
    let database_url = String::from(matches.value_of("database").unwrap());
    let useragent = String::from(matches.value_of("useragent").unwrap());

    println!("DATABASE_URL  : {}", database_url);
    println!("LOOP          : {}", do_loop);
    println!("SOURCE        : {}", source);
    println!("CONCURRENCY   : {}", concurrency);
    println!("STATIONS      : {}", check_stations);
    println!("PAUSE_SECONDS : {}", pause_seconds);
    println!("TCP_TIMEOUT   : {}", tcp_timeout);
    println!("MAX_DEPTH     : {}", max_depth);
    println!("RETRIES       : {}", retries);
    println!("DELETE        : {}", delete);
    println!("FAVICON       : {}", favicon);
    println!("USERAGENT     : {}", useragent);

    let database_url2 = database_url.clone();
    let source2 = source.clone();
    thread::spawn(move || loop {
        let conn = db::new(&database_url2);
        match conn {
            Ok(conn) => {
                let checks_hour = db::get_checks(&conn, 1, &source2);
                let checks_day = db::get_checks(&conn, 24, &source2);
                let stations_broken = db::get_station_count_broken(&conn);
                let stations_working = db::get_station_count_working(&conn);
                let stations_todo = db::get_station_count_todo(&conn, 24);
                let stations_deletable_never_worked =
                    db::get_deletable_never_working(&conn, 24 * 3);
                let stations_deletable_were_working =
                    db::get_deletable_were_working(&conn, 24 * 30);
                if delete {
                    db::delete_never_working(&conn, 24 * 3);
                    db::delete_were_working(&conn, 24 * 30);
                    db::delete_old_checks(&conn, 24 * 30);
                    db::delete_old_clicks(&conn, 24 * 30);
                }

                println!("STATS: {} Checks/Hour, {} Checks/Day, {} Working stations, {} Broken stations, {} to do, deletable {} + {}", checks_hour, checks_day, stations_working, stations_broken, stations_todo, stations_deletable_never_worked, stations_deletable_were_working);
            }
            Err(e) => {
                println!("Database connection error {}", e);
            }
        }
        thread::sleep(Duration::from_secs(3600));
    });

    loop {
        if verbosity > 0 {
            println!("new batch");
        }

        let checked_count = check::dbcheck(
            &database_url,
            &source,
            concurrency,
            check_stations,
            &useragent,
            tcp_timeout,
            max_depth,
            retries,
            favicon,
            verbosity,
        );
        if !do_loop {
            break;
        }

        if checked_count == 0 {
            if verbosity > 0 {
                println!("pause for {} secs", pause_seconds);
            }
            thread::sleep(Duration::from_secs(pause_seconds));
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    }
}
