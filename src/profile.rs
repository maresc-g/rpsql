use std::fs;
use std::io::{self, Write};
use std::path;
use dirs;
use std::env;
use crate::parse_args::ConnectionOptions;

const PROFILES_DIR: &str = "rpsql/profiles";

pub fn choose() -> io::Result<()> {
    let dir = _get_config_dir();
    let entries = fs::read_dir(_get_config_dir());
    let mut profiles: Vec<path::PathBuf> = Vec::new();

    match entries {
        Ok(all_entries) => {
            profiles = _get_profile_files(all_entries)?;
        },
        Err(err) => {
            _create_dir_if_not_found(err, &dir)?;
        }
    }
    println!("(0) Create new profile");
    for (i, profile) in profiles.iter().enumerate() {
        println!("({}) {}", i + 1, profile.file_name().unwrap_or_default().to_str().unwrap());
    }
    let mut buffer = String::new();
    print!("Choose your profile : ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buffer)?;
    let choice = buffer.trim();
    match choice {
        "0" => {
            println!("Creating new profile");
            let (profile_name, connect_options) = _create_new_profile();
            _save_profile(&dir, &profile_name, &connect_options)?;
        }
        _ => {
            println!("Using profile {}", choice);
        }
    }
    Ok(())
}

fn _create_new_profile() -> (String, ConnectionOptions) {
    let mut c = ConnectionOptions::new();
    let username = env::var("USER").unwrap_or_else(|_| String::from("postgres"));
    let profile_name = _read_attribute("Profile name (.json is added automatically)", None);
    c.host = _read_attribute("Host", Some("localhost".to_string()));
    c.port = _read_attribute("Port", Some("5432".to_string()));
    c.dbname = _read_attribute("Database name", Some(username.clone()));
    c.user = _read_attribute("User", Some(username.clone()));
    (profile_name, c)
}

fn _read_attribute(prompt: &str, default: Option<String>) -> String {
    let mut buffer = String::new();
    let mut default_val = String::new();
    let mut prompt_with_default = prompt.to_string();
    if let Some(d) = &default {
        default_val = d.clone();
        prompt_with_default.push_str(format!(" (Default = {})", default_val).as_str());
    }
    while buffer.is_empty() {
        print!("{} : ", prompt_with_default);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).unwrap_or_default();
        buffer = buffer.trim().to_string();
        if buffer.is_empty() && !default_val.is_empty() {
            buffer = default_val.clone();
        }
    }
    buffer
}

fn _save_profile(dir: &path::PathBuf, profile_name: &String, connect_options: &ConnectionOptions) -> io::Result<()> {
    let mut filename = path::PathBuf::new();
    filename.push(dir);
    filename.push(format!("{}.json", profile_name));
    filename.set_extension("json");
    let mut file = fs::File::create(filename)?;
    file.write_all(connect_options.to_json().as_bytes())?;
    Ok(())
}

fn _get_config_dir() -> path::PathBuf {
    let mut p = dirs::config_dir().unwrap_or_default();
    p.push(PROFILES_DIR);
    p
}

fn _get_profile_files(entries: fs::ReadDir) -> Result<Vec<path::PathBuf>, io::Error> {
    let mut profiles: Vec<path::PathBuf> = Vec::new();
    for entry in entries {
        let e = entry?;
        if !e.metadata()?.is_dir() {
            profiles.push(e.path());
        }
    }
    Ok(profiles)
}

fn _create_dir_if_not_found(err: io::Error, dir: &path::PathBuf) -> std::io::Result<()> {
    match err.kind() {
        io::ErrorKind::NotFound => {
            if let Err(err) = _create_dir(&dir) {
                eprintln!("Error while creating profiles directory ({}): {}, exiting", dir.to_str().unwrap(), err);
                Err(err)
            }
            else {
                Ok(())
            }
        }
        _ => {
            eprintln!("Error while accessing profiles directory ({}): {}, exiting", dir.to_str().unwrap(), err);
            Err(err)
        }
    }
}

fn _create_dir(path: &path::PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(path)
}
