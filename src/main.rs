mod parse_args;
mod profile;
mod connection_options;
mod sql;

use parse_args::Config;
use connection_options::ConnectionOptions;
use std::io;

fn main() {
    let config : Config = parse_args::parse();
    let res: Result<ConnectionOptions, io::Error>;

    match config {
        Config::None => {
            res = profile::choose();
        }
        Config::Profile(p) => {
            res = profile::load(&p);
        }
        Config::ConnectionOptions(c) => {
            res = Ok(c);
        }
    }

    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    if let Err(e) = sql::handle_connection(&res.unwrap()) {
        eprintln!("{}", e);
        std::process::exit(1);        
    }
}
