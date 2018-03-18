#![allow(non_snake_case)]
table! {
    Station (StationID) {
        StationID -> Integer,
        Name -> Text,
        Url -> Text,
        Bitrate -> Integer,
        Codec -> Varchar,
        UrlCache -> Text,
        LastCheckTime -> Timestamp,
    }
}