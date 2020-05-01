mod parse_args;
mod profile;
mod connection_options;
mod sql;
mod ui;
mod history;
mod pgpass;

use parse_args::Config;
use connection_options::ConnectionOptions;
use std::io::Error;
use termion::raw::IntoRawMode;
use history::History;
use pgpass::parse;
use dirs;
use ui::event_loop::{self, TermPos};

pub fn main_loop(connection_options: &ConnectionOptions, password: Option<String>) -> Result<(), String> {
    let mut client = sql::try_connect(&connection_options, password)?;
    event_loop::init();
    let mut again = true;
    let mut tp = TermPos::new();
    let mut stdout = std::io::stdout().into_raw_mode().unwrap();
    let mut history = History::load_from_file();

    while again {
        if let Some(query) = event_loop::get_input(&mut tp, &mut stdout, &mut history) {
            if !query.trim().is_empty() {
                let res = sql::handle_query(&mut client, query.as_str());
                if let Err(e) = res {
                    event_loop::display_string_on_new_line(&mut tp, &mut stdout, &e.to_string());
                }
                else {
                    event_loop::display_vec_on_new_line(&mut tp, &mut stdout, &res.unwrap());
                }
            }
        }
        else {
            again = false;
        }
    }

    Ok(())
}


fn main() {
    let config : Config = parse_args::parse();
    let res: Result<ConnectionOptions, Error> = match config {
        Config::None => profile::choose(),
        Config::Profile(p) => profile::load(&p),
        Config::ConnectionOptions(c) => Ok(c)
    };
    match res {
        Ok(options) => {
            let password = if let Some(ref mut home) = dirs::home_dir() {
                home.push(".pgpass");
                pgpass::parse(&home, &options)
            } else {
                None
            };

            if let Err(e) = main_loop(&options, password) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
