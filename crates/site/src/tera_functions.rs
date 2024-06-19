use std::collections::HashMap;

use content::page::Page;
use serde_json::Value;
use tera::{from_value, to_value, Error};

pub fn posts_in_series(args: &HashMap<String, Value>) -> Result<Value, Error> {
    let series = args
        .get("series")
        .ok_or("Function `posts_in_series` expected argument `series`")
        .map(|v| from_value::<String>(v.clone()))??;
    let posts = args
        .get("posts")
        .ok_or("Function `posts_in_series` expected argument `posts`")
        .map(|v| from_value::<Vec<Page>>(v.clone()))??;

    let posts_in_series = posts
        .into_iter()
        .filter(|p| p.path.starts_with(format!("public/series/{series}")))
        .collect::<Vec<Page>>();

    Ok(to_value(posts_in_series)?)
}

pub fn get_series_indexes(args: &HashMap<String, Value>) -> Result<Value, Error> {
    let indexes = args
        .get("indexes")
        .ok_or("Function `get_series_indexes` expected argument `indexes`")
        .map(|v| from_value::<Vec<Page>>(v.clone()))??;

    let series_indexes = indexes
        .into_iter()
        .filter(|p| {
            p.path.starts_with("public/series/") && !p.path.starts_with("public/series/index.html")
        })
        .collect::<Vec<Page>>();

    println!("{:#?}", series_indexes);

    Ok(to_value(series_indexes)?)
}
