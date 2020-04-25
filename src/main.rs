mod parse_args;
mod profile;
mod connection_options;

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
}
