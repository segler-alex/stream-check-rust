use diesel::prelude::*;
use std::env;

pub fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

use models::*;

pub fn get_stations(conn: MysqlConnection, itemcount: i64) -> Vec<StationItem> {
    use schema::Station::dsl::*;
    use diesel::dsl::*;
    let mut list = vec![];
    
    let result = Station
        .limit(itemcount)
        //.filter(LastCheckTime.gt(now))
        .order(LastCheckTime.asc())
        .load::<StationItem>(&conn)
        .expect("aaa");

    for station in result {
        list.push(station.clone());
    }
    list
}
