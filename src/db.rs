use std::error::Error;
use mysql;
use models::StationItem;
use models::StationCheckItemNew;

pub fn get_stations_to_check(pool: &mysql::Pool, hours: u32, itemcount: u32) -> Vec<StationItem> {
    let query = format!("SELECT StationID,StationUuid,Name,Url FROM Station WHERE LastCheckTime IS NULL OR LastCheckTime < NOW() - INTERVAL {} HOUR ORDER BY RAND() ASC LIMIT {}", hours, itemcount);
    get_stations_query(pool, query)
}

fn get_stations_query(pool: &mysql::Pool, query: String) -> Vec<StationItem> {
    let mut stations: Vec<StationItem> = vec![];
    let results = pool.prep_exec(query, ());
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();
            let s = StationItem {
                id:              row.take("StationID").unwrap(),
                uuid:            row.take("StationUuid").unwrap_or("".to_string()),
                name:            row.take("Name").unwrap_or("".to_string()),
                url:             row.take("Url").unwrap_or("".to_string()),
            };
            stations.push(s);
        }
    }

    stations
}

pub fn insert_check(pool: &mysql::Pool,item: &StationCheckItemNew){
    let query = String::from("INSERT INTO StationCheck(StationUuid,CheckUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache) VALUES(?,UUID(),?,?,?,?,?,NOW(),?)");
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute((&item.station_uuid,&item.source,&item.codec,&item.bitrate,&item.hls,&item.check_ok,&item.url));
    match result {
        Ok(_) => {},
        Err(err) => {println!("{}",err);}
    }
}

pub fn update_station(pool: &mysql::Pool,item: &StationCheckItemNew){
    let mut query: String = String::from("UPDATE Station SET LastCheckTime=NOW(),LastCheckOkTime=NOW(),LastCheckOk=?,Codec=?,Bitrate=?,UrlCache=? WHERE StationUuid=?");
    if !item.check_ok{
        query = format!("UPDATE Station SET LastCheckTime=NOW(),LastCheckOk=?,Codec=?,Bitrate=?,UrlCache=? WHERE StationUuid=?");
    }
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute((&item.check_ok,&item.station_uuid,&item.codec,&item.bitrate,&item.url));
    match result {
        Ok(_) => {},
        Err(err) => {println!("{}",err);}
    }
}

pub fn new(connection_str: &str) -> Result<mysql::Pool, Box<Error>> {
    let pool = mysql::Pool::new(connection_str)?;
    Ok(pool)
}
