use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::connection_options::ConnectionOptions;

pub fn parse<P: AsRef<Path>>(path: P, options: &ConnectionOptions) -> Option<String> {
    let file = open_file_if_safe(path)?;

    parse_file(file, options)
}

fn open_file_if_safe<P: AsRef<Path>>(path: P) -> Option<File> {
    match File::open(&path) {
        Ok(file) => {
            match file.metadata() {
                Ok(metadata) => {
                    let permissions = metadata.permissions();

                    if permissions.mode() & 0o777 > 0o0600 {
                        None
                    } else {
                        Some(file)
                    }
                },
                Err(err) => {
                    eprintln!("Couldn't inspect file permissions of '{}': {}", path.as_ref().display(), err);
                    None
                }
            }
        },
        Err(err) => {
            eprintln!("Couldn't open file '{}': {}", path.as_ref().display(), err);
            None
        }
    }
}

#[derive(Debug)]
struct Entry<'a> {
    hostname: &'a str,
    port: &'a str,
    database: &'a str,
    username: &'a str,
    password: &'a str,
}

impl Entry<'_> {
    fn parse(s: &str) -> Option<Entry> {
        let mut fields = s.split(':');
        let hostname = fields.next()?;
        let port = fields.next()?;
        let database = fields.next()?;
        let username = fields.next()?;
        let password = fields.next()?;

        Some(Entry{ hostname, port, database, username, password })
    }

    fn matches(&self, hostname: &str, port: &str, database: &str, username: &str) -> bool {
        (self.hostname == "*" || hostname == self.hostname)
            && (self.port == "*" || port == self.port)
            && (self.database == "*" || database == self.database)
            && (self.username == "*" || username == self.username)
    }
}

enum Line<'a> {
    Commented(&'a str),
    Empty(&'a str),
    Parsed(Entry<'a>),
    Malformed(&'a str),
}

impl Line<'_> {
    fn parse(s: &str) -> Line {
        let s = s.trim();

        if s.starts_with('#') {
            Line::Commented(s)
        } else if s.is_empty() {
            Line::Empty(s)
        } else if let Some(entry) = Entry::parse(s) {
            Line::Parsed(entry)
        } else {
            Line::Malformed(s)
        }
    }
}

fn parse_file(file: File, options: &ConnectionOptions) -> Option<String> {
    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(entry) = line {
            match Line::parse(&entry) {
                Line::Commented(_) => {},
                Line::Empty(_) => {},
                Line::Parsed(e) => {
                    if e.matches(&options.host, &options.port, &options.dbname, &options.user) {
                        return Some(e.password.to_string())
                    }
                },
                Line::Malformed(_) => {},
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_permissions() {
        let path = "tests/pgpass/bad_permissions.pgpass";

        if let Some(_file) = open_file_if_safe(path) {
            assert!(false, "Shouldn't have been able to open '{}'", path);
        } else {
            assert!(true);
        }
    }

    #[test]
    fn ok_permissions() {
        let path = "tests/pgpass/ok.pgpass";

        if let Some(_file) = open_file_if_safe(path) {
            assert!(true);
        } else {
            assert!(false, "Should have been able to open '{}'", path);
        }
    }

    #[test]
    fn rpsql_localhost_5432_rpsql() {
        let path = "tests/pgpass/ok.pgpass";
        let password = parse(path, &ConnectionOptions{
            dbname: "rpsql".to_string(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            user: "rpsql".to_string(),
            password: "".to_string()
        });

        assert_eq!(password, Some("defaultpass".to_string()))
    }
}
