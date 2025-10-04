use clap::Parser;
use std::fs;
use serde::{Deserialize};

#[derive(Parser, Debug)]
#[command(version, about = "Bitwarden to Mooltipass converter", long_about = None)]
struct Cli {
    // Bitwarden json file
    #[arg(short, long)]
    file: String,

    // Filter out a single folder
    #[arg(long)]
    filter: Option<String>,

    // Exclude a folder
    #[arg(short, long)]
    exclude: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Login {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Item {
    name: String,

    #[serde(rename = "folderId")]
    folder_id: Option<String>,

    #[serde(default)]
    login: Option<Login>, 
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    items: Vec<Item>,
}

fn load_json(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(file)?;
    let json_cfg: Config = serde_json::from_str(&data)?;
    Ok(json_cfg)
}

fn main() {
    let args = Cli::parse();

    println!("Hello, {}!", args.file);
    let cfg = load_json(&args.file);
    for item in cfg.unwrap().items {
        match item.folder_id {
            name => println!("{}", item.name),
            _ => println!("NO {}", item.name),
        }
    }
}
