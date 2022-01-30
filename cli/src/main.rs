//! A simple CLI interface for interacting with posts for my blog

#[macro_use]
extern crate dotenv_codegen;

extern crate dotenv;

use anyhow::Context;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io;

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Publish { summary: String, path: String },
    Delete { id: i32 },
    Fetch { id: i32 },
}

#[derive(Serialize)]
struct NewPost {
    summary: String,
    body: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    pub message: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    pub data: Value,
}

fn get_credentials() -> anyhow::Result<(String, String)> {
    let mut username = String::new();

    println!("Username:");
    io::stdin()
        .read_line(&mut username)
        .context("Problem while reading user input")?;

    println!("Password:");
    let pass = rpassword::read_password().context("Problem while reading user input")?;

    Ok((username.trim().to_string(), pass))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let api_url = dotenv!("CLI_API_URL");

    let args = Args::parse();

    let client = reqwest::Client::new();

    match args.command {
        Commands::Publish { summary, path } => {
            let (username, password) = get_credentials()?;

            let body = fs::read_to_string(path).context("Problem while reading file")?;
            let json = NewPost { summary, body };

            let response = client
                .post(api_url.to_owned() + "/api/posts")
                .json(&json)
                .header("username", &username)
                .header("password", &password)
                .send()
                .await?;

            if response.status() != 200 {
                let error_res = response.json::<ErrorResponse>().await?;
                println!("{}", error_res.message);
                std::process::exit(1);
            }

            println!("Succesfully Created Post");
        }
        Commands::Delete { id } => {
            let (username, password) = get_credentials()?;

            let response = client
                .delete(api_url.to_owned() + &format!("/api/posts/{}", id))
                .header("username", &username)
                .header("password", &password)
                .send()
                .await?;

            if response.status() != 200 {
                let error_res = response.json::<ErrorResponse>().await?;
                println!("{}", error_res.message);
                std::process::exit(1);
            }

            println!("Succesfully Deleted Post");
        }
        Commands::Fetch { id } => {
            let response = client
                .get(api_url.to_owned() + &format!("/api/posts/{}", id))
                .send()
                .await?;

            if response.status() != 200 {
                let error_res = response.json::<ErrorResponse>().await?;
                println!("{}", error_res.message);
                std::process::exit(1);
            }

            let json_res = response.json::<ApiResponse>().await?;
            println!("{:#?}", json_res.data);
        }
    }

    Ok(())
}
