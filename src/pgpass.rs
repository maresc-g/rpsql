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
struct Entry {
    hostname: String,
    port: String,
    database: String,
    username: String,
    password: String,
}

impl Entry {
    fn parse(s: &str) -> Option<Entry> {
        let mut fields = FieldSplit::new(s);
        let hostname = fields.next()?;
        let port = fields.next()?;
        let database = fields.next()?;
        let username = fields.next()?;
        let password = fields.next()?;

        if let Some(_) = fields.next() {
            None
        } else {
            Some(Entry{ hostname, port, database, username, password })
        }
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
    Parsed(Entry),
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

struct FieldSplit<'a> {
    s: &'a str,
}

impl<'a> FieldSplit<'a> {
    fn new(s: &'a str) -> FieldSplit<'a> {
        FieldSplit{ s }
    }
}

impl<'a> Iterator for FieldSplit<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut field = None;
        let mut backslash = false;
        let mut index = 0;

        for (i, c) in self.s.chars().enumerate() {
            index = i;
            field = field.or_else(|| {
                Some(String::new())
            });
            let field = field.as_mut().unwrap();
            match c {
                '\\' => {
                    if backslash {
                        field.push(c);
                        backslash = false;
                    } else {
                        backslash = true;
                    }
                },
                ':' => {
                    if backslash {
                        field.push(c);
                        backslash = false;
                    } else {
                        break;
                    }
                }
                _ => {
                    if backslash {
                        field.push('\\');
                        backslash = false;
                    }
                    field.push(c);
                },
            }
        }
        if index > 0 {
            self.s = &self.s[index + 1..];
        }
        if backslash {
            field.as_mut().unwrap().push('\\');
        }
        field
    }
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
        });

        assert_eq!(password, Some("defaultpass".to_string()))
    }

    #[test]
    fn field_iterator_easy() {
        let input = "one:two:three:four:five:six:seven";
        let fields: Vec<String> = FieldSplit::new(input).collect();

        assert_eq!(fields, vec!["one", "two", "three", "four", "five", "six", "seven"]);
    }

    #[test]
    fn field_iterator_escaped_colon() {
        let input = r"one\:two:three:four:five";
        let fields: Vec<String> = FieldSplit::new(input).collect();

        assert_eq!(fields, vec!["one:two", "three", "four", "five"]);
    }

    #[test]
    fn field_iterator_escaped_backslash() {
        let input = r"one:two\\:three:four:five";
        let fields: Vec<String> = FieldSplit::new(input).collect();

        assert_eq!(fields, vec!["one", r"two\", "three", "four", "five"]);
    }

    #[test]
    fn field_iterator_escaped_mess() {
        let input = r"\one\:two\\:three:four\\\:five\";
        let fields: Vec<String> = FieldSplit::new(input).collect();

        assert_eq!(fields, vec![r"\one:two\", "three", r"four\:five\"]);
    }
}
