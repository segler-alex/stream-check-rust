#![allow(non_snake_case)]
use chrono::NaiveDateTime;

#[derive(Queryable,Clone)]
pub struct StationItem {
    pub StationID: i32,
    pub Name: String,
    pub Url: String,
    pub Bitrate: i32,
    pub Codec: String,
    pub UrlCache: String,
    pub LastCheckTime: NaiveDateTime,
}