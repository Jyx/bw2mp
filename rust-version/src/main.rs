//! # Bitwarden to Mooltipass Converter
//!
//! This program converts a Bitwarden JSON export into a CSV format suitable for Mooltipass.
//! It reads login credentials from the JSON file and outputs them as CSV lines with format:
//! `uri,username,password`
//!
//! ## Usage
//! Run with: `cargo run -- --file path/to/bitwarden.json`
//!
//! ## Learning Rust Concepts
//! This code demonstrates:
//! - Command-line argument parsing with `clap`
//! - JSON deserialization with `serde`
//! - Error handling with `Result` and `?` operator
//! - Ownership and borrowing in Rust
//! - Iterators and closures for data processing

use clap::Parser;
use serde::Deserialize;
use std::fs;

/// Command-line arguments for the application.
/// This struct defines what options the user can pass when running the program.
/// Clap automatically generates help text and parses the arguments.
#[derive(Parser, Debug)]
#[command(version, about = "Bitwarden to Mooltipass converter", long_about = None)]
struct Cli {
    /// Path to the Bitwarden exported JSON file.
    /// This is required and contains the password data to convert.
    #[arg(short, long)]
    file: String,

    /// Filter to include only items from a specific folder by exact name.
    /// If provided, only items in this folder will be processed.
    #[arg(long)]
    filter: Option<String>,

    /// Exclude items from a specific folder by exact name.
    /// Items in this folder will be skipped.
    #[arg(short, long)]
    exclude: Option<String>,
}

/// Represents a URI (website address) associated with a login.
/// Each login can have multiple URIs.
#[derive(Debug, Deserialize, Clone)]
struct Uri {
    /// The actual URI string, like "https://example.com".
    uri: String,
}

/// Represents login credentials for a website.
/// Contains username, password, and associated URIs.
#[derive(Debug, Deserialize, Clone)]
struct Login {
    /// The username for the login.
    username: String,
    /// The password for the login.
    password: String,

    /// List of URIs where this login can be used.
    /// Defaults to an empty list if not present in JSON.
    #[serde(default)]
    uris: Vec<Uri>,
}

/// Represents a folder in Bitwarden.
/// Folders organize items (logins).
#[derive(Debug, Deserialize)]
struct Folder {
    /// Unique ID of the folder.
    id: String,
    /// Human-readable name of the folder.
    name: String,
}

/// Represents an item (usually a login) in Bitwarden.
/// Items can be in folders and contain login data.
#[derive(Debug, Deserialize)]
struct Item {
    /// ID of the folder this item belongs to, if any.
    /// Uses "folderId" from JSON.
    #[serde(rename = "folderId")]
    folder_id: Option<String>,

    /// The login data for this item, if it exists.
    /// Defaults to None if not present.
    #[serde(default)]
    login: Option<Login>,
}

/// The top-level structure of the Bitwarden JSON export.
/// Contains all folders and items.
#[derive(Debug, Deserialize)]
struct Config {
    /// List of all folders in the export.
    /// Defaults to empty if not present.
    #[serde(default)]
    folders: Vec<Folder>,

    /// List of all items (logins) in the export.
    /// Defaults to empty if not present.
    #[serde(default)]
    items: Vec<Item>,
}

/// Loads and parses the Bitwarden JSON file.
/// Reads the file as a string, then deserializes it into our Config struct.
/// Returns an error if the file can't be read or the JSON is invalid.
fn load_json(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(file)?;
    let json_cfg: Config = serde_json::from_str(&data)?;
    Ok(json_cfg)
}

/// Finds the folder ID by exact name match.
/// Searches through the list of folders and returns the ID of the first one
/// whose name exactly matches the given name.
/// Returns None if no folder with that name is found.
fn find_folder_id_by_name(folders: &[Folder], name: &str) -> Option<String> {
    folders
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.id.clone())
}

/// The main entry point of the program.
/// Parses command-line arguments, loads the JSON, processes items,
/// and outputs to stdout and CSV.
///
/// This function demonstrates Rust's error handling patterns:
/// - Using `match` to handle `Result` types
/// - Early returns on errors (similar to exceptions but more explicit)
/// - Borrowing strings with `&` to avoid copying
fn main() {
    // Parse command-line arguments using Clap
    let args = Cli::parse();

    // Print the program start message to stdout
    println!("Bitwarden to Mooltipass");

    // Clap already ensures the file argument is provided, but we double-check
    // In Rust, strings are checked for emptiness with .is_empty()
    if args.file.is_empty() {
        eprintln!("Error: Need a json file from Bitwarden");
        return;
    }

    // Load the JSON configuration from the file
    // The `match` expression handles the Result returned by load_json
    // - Ok(config) means success, we get the Config struct
    // - Err(e) means failure, we print the error and exit
    // The `&args.file` borrows the string to avoid moving it
    let cfg = match load_json(&args.file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load JSON: {}\n{}", &args.file, e);
            return;
        }
    };

    // Find folder IDs for filter and exclude by exact name match
    // This demonstrates Rust's Option chaining with `and_then`
    // - `args.filter.as_ref()` borrows the Option<String> as Option<&String>
    // - `and_then` only calls the closure if the Option has a value
    // - The closure calls our helper function with borrowed references
    let filter_id = args
        .filter
        .as_ref()
        .and_then(|name| find_folder_id_by_name(&cfg.folders, name));
    let exclude_id = args
        .exclude
        .as_ref()
        .and_then(|name| find_folder_id_by_name(&cfg.folders, name));



    // Create the CSV output file (same name as input with .csv extension)
    // `format!` creates a String, similar to f-strings in Python
    // `fs::File::create` returns a Result<File, Error>
    // We use `mut` because we'll write to the file later
    let csv_path = format!("{}.csv", args.file);
    let mut csv_file = match fs::File::create(&csv_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create CSV file: {}\n{}", csv_path, e);
            return;
        }
    };

    // Import the Write trait so we can call write_all on the file
    // In Rust, traits must be in scope to use their methods
    use std::io::Write;

    // Process each item in the configuration
    // `&cfg.items` borrows the vector, giving us `&Item` references
    // This avoids copying the items and is more efficient
    for item in &cfg.items {
        // Get the folder ID of this item, if any
        // `as_ref()` converts `&Option<String>` to `Option<&String>`
        let folder_id = item.folder_id.as_ref();

        // Skip this item if it matches the exclude folder
        // `if let` is pattern matching - if exclude_id has a value AND folder_id equals it
        if let Some(ex_id) = &exclude_id && folder_id == Some(ex_id) {
            continue;
        }

        // Determine if this item should be included
        // Include if no filter is set, or if the item's folder matches the filter
        // This shows conditional logic with Options
        let include = if let Some(f_id) = &filter_id {
            folder_id == Some(f_id)
        } else {
            true
        };

        // Skip if not included
        if !include {
            continue;
        }

        // If the item has login data, process its URIs
        // `if let` pattern matches on the Option
        if let Some(login_data) = &item.login {
            // Clone the login data to avoid borrowing issues in the inner loop
            // In Rust, we can't borrow from login_data while also borrowing uri.uri
            // Cloning creates owned copies we can reference freely
            let login = login_data.clone();

            // Iterate over each URI for this login
            // `login.uris` is a Vec<Uri>, so we get each Uri by value
            for uri in login.uris {
                // Format the output line: uri,username,password
                // Similar to Python's f-strings, but with `{}` placeholders
                let line = format!("{},{},{}\n", &uri.uri, &login.username, &login.password);

                // Print to stdout (without newline since line already has it)
                print!("{}", line);

                // Write to CSV file
                // `write_all` takes `&[u8]`, so we convert the string to bytes
                // `as_bytes()` borrows the string as a byte slice
                if let Err(e) = csv_file.write_all(line.as_bytes()) {
                    eprintln!("Failed to write to CSV: {}", e);
                    return;
                }
            }
        }
    }
}
