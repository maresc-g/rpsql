mod parse_args;

use parse_args::Config;

fn main() {
    let config : Config = parse_args::parse();
}
