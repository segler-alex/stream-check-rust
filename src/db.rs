use std::env;
use std::error::Error;
use mysql;
use models::StationItem;

pub fn get_stations(pool: &mysql::Pool, itemcount: u32) -> Vec<StationItem> {
    let query = format!("SELECT StationID,StationUuid,Name,Url FROM Station WHERE LastCheckOkTime < NOW() - INTERVAL 1 DAY ORDER BY RAND() ASC LIMIT {}", itemcount);
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

pub fn new() -> Result<mysql::Pool, Box<Error>> {
    let database_url = env::var("DATABASE_URL")?;
    let connection_string = database_url.clone();
    println!("Connection string: {}", connection_string);

    let pool = mysql::Pool::new(connection_string)?;
    Ok(pool)
}
