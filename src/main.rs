mod parse_args;
mod profile;

use parse_args::Config;

fn main() {
    let config : Config = parse_args::parse();

    match config {
        Config::None => {
            if let Err(_) = profile::choose() {
                std::process::exit(1);
            }
        }
        Config::Profile(_) => {

        }
        Config::ConnectionOptions(_) => {

        }
    }
}
