use std::collections::HashMap;

pub struct PlaylistItem {
    pub title: String,
    pub url: String,
}

pub fn decode_playlist(content: &str) -> Vec<PlaylistItem> {
    let lines = content.lines();
    let mut list = vec![];


    list
}
