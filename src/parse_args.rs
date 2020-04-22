use clap::{Arg, ArgGroup, App};
use std::env;

#[derive(Debug)]
pub enum Config {
    None,
    Profile(String),
    ConnectionOptions(ConnectionOptions),
}

#[derive(Debug)]
pub struct ConnectionOptions {
    pub dbname: String,
    pub host: String,
    pub port: String,
    pub user: String,
}

impl ConnectionOptions {
    pub fn to_connection_string(&self) -> String {
        format!("host={} port={} user={} dbname={}", self.host, self.port, self.user, self.dbname)
    }
}

pub fn parse() -> Config {
    let matches = App::new("rpsql")
                    .version("0.1.0")
                    .author("Guillaume M.")
                    .about("Alternative to psql written in rust")
                    .arg(Arg::with_name("dbname")
                        .short('d')
                        .long("dbname")
                        .help("Database to connect to")
                        .takes_value(true))
                    .arg(Arg::with_name("host")
                        .short('h')
                        .long("host")
                        .help("Database server hostname")
                        .takes_value(true))
                    .arg(Arg::with_name("port")
                        .short('p')
                        .long("port")
                        .help("Database server port")
                        .takes_value(true))
                    .arg(Arg::with_name("user")
                        .short('u')
                        .long("user")
                        .help("Database user")
                        .takes_value(true))
                    .arg(Arg::with_name("profile")
                        .short('P')
                        .long("profile")
                        .help("Connection profile, a shortcut instead of using -d -h -p -u")
                        .takes_value(true)
                        .conflicts_with_all(&["dbname", "host", "port", "user"]))
                    .group(ArgGroup::with_name("connection_options")
                        .args(&["dbname", "host", "port", "user"])
                        .multiple(true))
                .get_matches();
    let mut result = Config::None;
    if matches.is_present("profile") {
        result = Config::Profile(matches.value_of("profile").unwrap().to_string());
    }
    else if matches.is_present("connection_options") {
        let username = env::var("USER").unwrap_or_else(|_| String::from("postgres"));
        result = Config::ConnectionOptions(ConnectionOptions{
            dbname : matches.value_of("dbname").unwrap_or_else(|| &username).to_string(),
            host : matches.value_of("host").unwrap_or_else(|| "localhost").to_string(),
            port : matches.value_of("port").unwrap_or_else(|| "5432").to_string(),
            user : matches.value_of("user").unwrap_or_else(|| &username).to_string(),
        });
    }
    println!("{:?}", result);
    result
}
