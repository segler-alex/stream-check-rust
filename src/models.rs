#![allow(non_snake_case)]
use chrono::NaiveDateTime;
use schema::StationCheck;

#[derive(Queryable,Clone)]
pub struct StationItem {
    pub StationID: i32,
    pub StationUuid: String,
    pub Name: String,
    pub Url: String,
    pub Bitrate: i32,
    pub Codec: String,
    pub UrlCache: String,
    pub LastCheckTime: NaiveDateTime,
}

#[derive(Queryable,Clone)]
pub struct StationCheckItem {
    pub CheckID: i32,
    pub StationUuid: String,
    pub CheckUuid: String,
    pub Source: String,
    pub Codec: String,
    pub Bitrate: i32,
    pub Hls: bool,
    pub CheckOK: bool,
    pub CheckTime: NaiveDateTime,
}


#[derive(Deserialize, Insertable)]
#[table_name="StationCheck"]
pub struct NewStationCheckItem<'a> {
    pub StationUuid: &'a str,
    pub CheckUuid: &'a str,
    pub Source: &'a str,
    pub Codec: &'a str,
    pub Bitrate: i32,
    pub Hls: bool,
    pub CheckOK: bool,
}