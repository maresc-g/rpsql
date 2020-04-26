mod parse_args;
mod profile;
mod connection_options;
mod sql;
mod ui;

use parse_args::Config;
use connection_options::ConnectionOptions;
use std::io::Error;

pub fn main_loop(connection_options: &ConnectionOptions) -> Result<(), String> {
    ui::init();
    let mut again = true;
    while again {
        if let Ok(_) = ui::get_input() {
        }
        else {
            again = false;
        }
    }
    // let mut client = sql::try_connect(&connection_options)?;
    //
    // let mut again = true;
    // while again {
    //     let res = _handle_input();
    //     if let Err(e) = res {
    //         eprintln!("{}", e);
    //         again = false;
    //     }
    //     else {
    //         let query = res.unwrap();
    //         if query.is_empty() {
    //             again = false;
    //         }
    //         else if !query.trim().is_empty() {
    //             let res = sql::handle_query(&mut client, query.as_str());
    //             if let Err(e) = res {
    //                 eprintln!("{}", e);
    //             }
    //         }
    //     }
    // }

    Ok(())
}


fn main() {
    let config : Config = parse_args::parse();
    let res: Result<ConnectionOptions, Error>;

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

    if let Err(e) = main_loop(&res.unwrap()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
