use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;

pub fn parse(path: Option<&str>) -> Option<String> {
    let path = path.unwrap_or("/home/mestag_a/.pgpass");
    let file = open_file_if_safe(path)?;

    parse_file(file)
}

fn open_file_if_safe(path: &str) -> Option<File> {
    match File::open(path) {
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
                    eprintln!("Couldn't inspect file permissions of '{}': {}", path, err);
                    None
                }
            }
        },
        Err(err) => {
            eprintln!("Couldn't open file '{}': {}", path, err);
            None
        }
    }
}

#[derive(Debug)]
struct Entry {
    hostname: String,
    port: String,
    database: String,
    username: String,
    password: String,
}

impl Entry {
    fn parse(s: &str) -> Option<Entry> {
        let mut fields = s.split(':');
        let hostname = fields.next()?.to_string();
        let port = fields.next()?.to_string();
        let database = fields.next()?.to_string();
        let username = fields.next()?.to_string();
        let password = fields.next()?.to_string();

        Some(Entry{ hostname, port, database, username, password })
    }

    fn matches(&self, hostname: &str, port: &str, database: &str, username: &str) -> bool {
        (self.hostname == "*" || hostname == self.hostname)
            && (self.port == "*" || port == self.port)
            && (self.database == "*" || database == self.database)
            && (self.username == "*" || username == self.username)
    }
}

fn parse_file(file: File) -> Option<String> {
    let reader = BufReader::new(file);
    let uncommented_lines = reader.lines()
        .filter(|entry| {
            !entry.as_ref().unwrap().starts_with('#')
        });

    for line in uncommented_lines {
        if let Ok(entry) = line {
            let e = Entry::parse(&entry);

            match Entry::parse(&entry) {
                Some(entry) => {
                    println!("{:?}", e);
                },
                None => {
                    eprintln!("Ignoring entry '{}'", entry);
                }
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
    fn test() {
        parse(None);
    }
}