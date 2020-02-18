#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::{LevelFilter, Config, TerminalMode};


fn main() {
    let matches = clap_app!(todo =>
        (version: "0.1.0")
        (author: "Oliver Downard")
        (about: "A git tool to find TODOs in your commit")
        (@arg verbose: -v ... "Enable verbose mode")
    ).get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 | _ => LevelFilter::Trace,
    };

    simplelog::TermLogger::init(log_level, Config::default(), TerminalMode::Stdout).unwrap();


}
