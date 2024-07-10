use std::{collections::HashMap, path::Path};

use config::Config;
use content::page::Page;
use serde_json::Value;
use tera::{from_value, to_value, Error, Function};

pub fn make_posts_in_series<T: AsRef<Path> + Sync + Send>(output_directory: T) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> Result<Value, Error> {
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
                .filter(|p| {
                    p.path
                        .starts_with(output_directory.as_ref().join("series").join(&series))
                })
                .collect::<Vec<Page>>();

            Ok(to_value(posts_in_series)?)
        },
    )
}

pub fn make_get_series_indexes<T: AsRef<Path> + Sync + Send>(output_directory: T) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> Result<Value, Error> {
            let indexes = args
                .get("indexes")
                .ok_or("Function `get_series_indexes` expected argument `indexes`")
                .map(|v| from_value::<Vec<Page>>(v.clone()))??;

            let series_indexes = indexes
                .into_iter()
                .filter(|p| {
                    p.path.starts_with(output_directory.as_ref().join("series"))
                        && !p.path.starts_with(
                            output_directory.as_ref().join("series").join("index.html"),
                        )
                })
                .collect::<Vec<Page>>();

            Ok(to_value(series_indexes)?)
        },
    )
}
