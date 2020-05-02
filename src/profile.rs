use std::fs::{self, File, ReadDir};
use std::io::{self, Write};
use std::path;
use std::env;
use crate::connection_options::ConnectionOptions;

const PROFILES_DIR: &str = "rpsql/profiles";

pub fn choose() -> Result<ConnectionOptions, io::Error> {
    let (dir, profiles) = _get_dir_and_profiles()?;

    println!("(0) Create new profile");
    for (i, profile) in profiles.iter().enumerate() {
        let tmp = profile.with_extension("");
        println!("({}) {}", i + 1, tmp.file_name().unwrap_or_default().to_str().unwrap());
    }
    _get_user_choice(&dir, &profiles)
}

pub fn load(profile_name: &str) -> Result<ConnectionOptions, io::Error> {
    let (dir, profiles) = _get_dir_and_profiles()?;
    let mut profile_file = path::PathBuf::new();
    profile_file.push(&dir);
    profile_file.push(format!("{}.json", profile_name));
    if let Some(p) = profiles.iter().find(|&p| p == &profile_file) {
        _load_profile(p)
    }
    else {
        Err(io::Error::new(io::ErrorKind::NotFound, format!("Error : profile {} not found", profile_name)))
    }
}

fn _get_dir_and_profiles() -> Result<(path::PathBuf, Vec<path::PathBuf>), io::Error> {
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
    Ok((dir, profiles))
}

fn _get_user_choice(dir: &path::PathBuf, profiles: &[path::PathBuf]) -> Result<ConnectionOptions, io::Error> {
    loop {
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
                return Ok(connect_options);
            }
            _ => {
                println!("Using profile {}", choice);
                let mut profile_name = path::PathBuf::new();
                if let Ok(i) = choice.parse::<usize>() {
                    if i < 1 || i > profiles.len() + 1 {
                        eprintln!("Invalid choice");
                        continue;
                    }
                    profile_name = profiles[i - 1].clone();
                }
                else {
                    profile_name.push(&dir);
                    profile_name.push(format!("{}.json", choice));
                    if profiles.iter().find(|&p| p == &profile_name).is_none() {
                        eprintln!("Invalid choice");
                        continue;
                    }
                }
                return _load_profile(&profile_name);
            }
        }
    }
}

fn _create_new_profile() -> (String, ConnectionOptions) {
    let mut c = ConnectionOptions::default();
    let username = env::var("USER").unwrap_or_else(|_| String::from("postgres"));
    let profile_name = _read_attribute("Profile name (.json is added automatically)", None);
    c.host = _read_attribute("Host", Some("localhost".to_string()));
    c.port = _read_attribute("Port", Some("5432".to_string()));
    c.dbname = _read_attribute("Database name", Some(username.clone()));
    c.user = _read_attribute("User", Some(username));
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

fn _save_profile(dir: &path::PathBuf, profile_name: &str, connect_options: &ConnectionOptions) -> io::Result<()> {
    let mut filename = path::PathBuf::new();
    filename.push(dir);
    filename.push(format!("{}.json", profile_name));
    let mut file = File::create(filename)?;
    file.write_all(serde_json::to_string(&connect_options).unwrap().as_bytes())?;
    Ok(())
}

fn _load_profile(filename: &path::PathBuf) -> Result<ConnectionOptions, io::Error> {
    let contents = fs::read_to_string(filename)?;
    Ok(serde_json::from_str(&contents).unwrap())
}

fn _get_config_dir() -> path::PathBuf {
    let mut p = dirs::config_dir().unwrap_or_default();
    p.push(PROFILES_DIR);
    p
}

fn _get_profile_files(entries: ReadDir) -> Result<Vec<path::PathBuf>, io::Error> {
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
                Err(io::Error::new(io::ErrorKind::NotFound, format!("Error while creating profiles directory ({}): {}, exiting", dir.to_str().unwrap(), err)))
            }
            else {
                Ok(())
            }
        }
        _ => {
            Err(io::Error::new(io::ErrorKind::NotFound, format!("Error while accessing profiles directory ({}): {}, exiting", dir.to_str().unwrap(), err)))
        }
    }
}

fn _create_dir(path: &path::PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(path)
}
