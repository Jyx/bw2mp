//! # Bitwarden to Mooltipass Converter
//!
//! This program converts a Bitwarden JSON export into a CSV format suitable for Mooltipass.
//! It reads login credentials from the JSON file and outputs them as CSV lines with format:
//! `uri,username,password`
//!
//! ## Usage
//! Run with: `cargo run -- --in path/to/bitwarden.json`

use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::io::Write;

/// Command-line arguments for the application.
#[derive(Parser, Debug)]
#[command(version, about = "Bitwarden to Mooltipass converter", long_about = None)]
struct Cli {
    /// Path to the Bitwarden exported JSON file.
    #[arg(short = 'i', long = "in")]
    file: String,

    /// Regex pattern to match folder names for inclusion.
    #[arg(short = 'm', long = "match")]
    pattern: Option<String>,

    /// Regex pattern to match folder names for exclusion.
    #[arg(short = 'e', long = "exclude")]
    exclude: Option<String>,
}

/// Represents a URI (website address) associated with a login.
/// Each login can have multiple URIs.
#[derive(Debug, Deserialize, Clone)]
struct Uri {
    uri: String,
}

/// Represents login credentials for a website.
/// Contains username, password, and associated URIs.
#[derive(Debug, Deserialize, Clone)]
struct Login {
    username: String,
    password: String,

    #[serde(default)]
    uris: Vec<Uri>,
}

/// Represents a folder in Bitwarden.
/// Folders organize items (logins).
#[derive(Debug, Deserialize)]
struct Folder {
    id: String,
    name: String,
}

/// Represents an item (usually a login) in Bitwarden.
/// Items can be in folders and contain login data.
#[derive(Debug, Deserialize)]
struct Item {
    #[serde(rename = "folderId")]
    folder_id: Option<String>,

    #[serde(default)]
    login: Option<Login>,
}

/// The top-level structure of the Bitwarden JSON export.
/// Contains all folders and items.
#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    folders: Vec<Folder>,

    #[serde(default)]
    items: Vec<Item>,
}

/// Loads and parses the Bitwarden JSON file.
/// Returns an error if the file can't be read or the JSON is invalid.
fn load_json(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(file)?;
    let json_cfg: Config = serde_json::from_str(&data)?;
    Ok(json_cfg)
}

/// Finds the folder name by ID.
/// Returns the name of the folder with the given ID, or None if not found.
fn find_folder_name_by_id(folders: &[Folder], id: &str) -> Option<String> {
    folders.iter().find(|f| f.id == id).map(|f| f.name.clone())
}

/// The main entry point of the program.
/// Parses command-line arguments, loads the JSON, processes items,
/// and outputs to stdout and CSV.
fn main() {
    let args = Cli::parse();

    println!("Bitwarden to Mooltipass");

    let cfg = match load_json(&args.file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load JSON: {}\n{}", &args.file, e);
            return;
        }
    };

    let match_regex = if let Some(pattern) = &args.pattern {
        match Regex::new(pattern) {
            Ok(re) => Some(re),
            Err(e) => {
                eprintln!("Invalid include regex pattern: {}", e);
                return;
            }
        }
    } else {
        None
    };

    let exclude_regex = if let Some(pattern) = &args.exclude {
        match Regex::new(pattern) {
            Ok(re) => Some(re),
            Err(e) => {
                eprintln!("Invalid exclude regex pattern: {}", e);
                return;
            }
        }
    } else {
        None
    };

    let csv_path = format!("{}.csv", args.file);
    let mut csv_file = match fs::File::create(&csv_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create CSV file: {}\n{}", csv_path, e);
            return;
        }
    };

    for item in &cfg.items {
        let folder_name = item
            .folder_id
            .as_ref()
            .and_then(|id| find_folder_name_by_id(&cfg.folders, id))
            .unwrap_or_default();

        let include = {
            let matches_include = match_regex.as_ref().is_none_or(|re| re.is_match(&folder_name));
            let matches_exclude = exclude_regex.as_ref().is_some_and(|re| re.is_match(&folder_name));
            matches_include && !matches_exclude
        };

        if !include {
            continue;
        }

        if let Some(login_data) = &item.login {
            let login = login_data.clone();

            for uri in login.uris {
                let line = format!("{},{},{}\n", &uri.uri, &login.username, &login.password);

                print!("{}", line);

                if let Err(e) = csv_file.write_all(line.as_bytes()) {
                    eprintln!("Failed to write to CSV: {}", e);
                    return;
                }
            }
        }
    }
}
