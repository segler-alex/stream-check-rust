#![allow(non_snake_case)]
table! {
    Station (StationID) {
        StationID -> Integer,
        StationUuid -> Varchar,
        Name -> Text,
        Url -> Text,
        Bitrate -> Integer,
        Codec -> Varchar,
        UrlCache -> Text,
        LastCheckTime -> Timestamp,
    }
}

table! {
    StationCheck (CheckID) {
        CheckID -> Integer,
        StationUuid -> Varchar,
        CheckUuid -> Varchar,
        Source -> Varchar,
        Codec -> Varchar,
        Bitrate -> Integer,
        Hls -> Bool,
        CheckOK -> Bool,
        CheckTime -> Timestamp,
    }
}