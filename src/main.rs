use failure::Error;
use serde_json::Value;
#[macro_use]
extern crate log;
use reqwest::{header, Client, Response};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use structopt::StructOpt;

const base_url: &str = "https://api.github.com/user/starred";

/// Another Useless Github Utility
#[derive(StructOpt, Debug)]
#[structopt(name = "augu")]
struct CLI {
    /// The number of stars maximum to add each iteration
    #[structopt(short, default_value = "5")]
    num: i64,
    /// OAuth2 token used for the authentification (found in the Github options)
    #[structopt(short)]
    token: String,
    /// Path to the file containing the list of repositories to star (one per line) in this format : user/repository_name
    #[structopt(short, parse(from_os_str))]
    path: PathBuf,
}

type Result<T> = ::std::result::Result<T, Error>;

fn main() -> Result<()> {
    env_logger::init();
    // Retrieve secrets and configurations
    let config: CLI = CLI::from_args();
    let file = File::open(config.path.clone())?;
    let mut buf_reader = BufReader::new(file);
    let mut to_star = String::new();
    buf_reader.read_to_string(&mut to_star)?;

    // Header attached to the client for authentification
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("{} {}", "token", &config.token))
            .expect("It seems that your OAuth token is invalid!"),
    );
    let client = Client::builder().default_headers(headers).build()?;

    // Retrieve the list of starred repos
    let starred: Vec<String> = check_stars(&client)?;
    println!("Repositories starred fetched : {:?}", starred);

    // Every time the sum of the already starred repos and the number of lines of the list of repos to star is
    // a multiple of the one set in the configuration
    let lines = to_star.lines();
    let to_stars_len = lines.clone().count();
    println!("Sum : {}", to_stars_len + starred.len());
    if (starred.len() + to_stars_len) % config.num as usize == 0 {
        let _: Vec<_> = lines
            .inspect(|l| println!("Ligne : {:?}", l))
            .map(|repo| star(&client, repo).status())
            .inspect(|s| println!("Status : {:?}", s))
            .filter(reqwest::StatusCode::is_success)
            .collect();
    }
    Ok(())
}

fn star(client: &Client, repos: &str) -> Response {
    let url = format!("{}/{}", base_url, repos);
    println!("Parsed result for the repository to add : {:?}", repos);
    client
        .put(&url)
        .header(header::CONTENT_LENGTH, 0)
        .send()
        .expect("Could not reach the server")
}

fn deserialize(info: &str) -> Vec<String> {
    let json: Value =
        serde_json::from_str(info).expect("Malformed JSON response from the Github request");
    json.as_array()
        .expect("Impossible to convert the JSON to an array")
        .iter()
        .map(|repos| {
            repos["full_name"]
                .as_str()
                .expect("Impossible to convert the JSON to a string litteral")
                .to_owned()
        })
        .collect()
}

fn check_stars(client: &Client) -> Result<Vec<String>> {
    let user_stars = client.get(base_url).send()?.text()?;
    Ok(user_stars
        .lines()
        .map(deserialize)
        .nth(0)
        .expect("No JSON?"))
}
