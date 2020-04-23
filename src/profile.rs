use std::fs;
use std::io::{self, Write};
use std::path;
use dirs;

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
        "0\n" => {
            println!("Creating new profile");
        }
        _ => {
            println!("Using profile {}", choice);
        }
    }
    Ok(())
}

fn _create_new_profile() {
    
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
