mod parse_args;
mod profile;

use parse_args::Config;

fn main() {
    let config : Config = parse_args::parse();

    match config {
        Config::None => {
            let res = profile::choose();
            if let Err(e) = res {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            else {
                println!("{:?}", res.unwrap());
            }
        }
        Config::Profile(_) => {

        }
        Config::ConnectionOptions(_) => {

        }
    }
}
