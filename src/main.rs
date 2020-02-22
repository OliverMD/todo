#[macro_use]
extern crate clap;
extern crate failure;
#[macro_use]
extern crate log;
extern crate simplelog;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process;
use std::str;

use failure::Error;
use git2::{Repository, StatusOptions};
use regex::Regex;
use simplelog::{Config, LevelFilter, TerminalMode};

struct TodoLine {
    line: String,
    filename: String,
    lineno: u32,
}

impl TodoLine {
    fn new(line: String, filename: String, lineno: u32) -> TodoLine {
        TodoLine {line, filename, lineno}
    }
}

fn main() -> Result<(), Error> {
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

    simplelog::TermLogger::init(log_level, Config::default(), TerminalMode::Stdout)?;

    let repo = match Repository::discover(Path::new(".")) {
        Err(err) => {
            error!("Could not find git repo: {}", err);
            process::exit(0x0001);
        },
        Ok(repo) => repo,
    };

    let mut todo_lines: Vec<TodoLine> = Vec::new();

    let re = Regex::new(r"^(.*)//(.*)[Tt][Oo][Dd][Oo](.*)")?;

    extract_for_existing_files(&repo, &mut todo_lines, |l| re.is_match(l))?;
    extract_for_new_files(&repo, &mut todo_lines, |l| re.is_match(l))?;


    for line in todo_lines {
        println!("{}:{} - {}", line.filename, line.lineno, line.line.trim_end());
    };

    Result::Ok(())
}

fn extract_for_existing_files<F: Fn(&str) -> bool>(repo: &Repository, results: &mut Vec<TodoLine>, is_todo_test: F) -> Result<(), Error> {
    let oid = repo.head()?.resolve()?.target().ok_or(failure::err_msg("Could not get Oid"))?;

    let tree = repo.find_commit(oid)?.tree()?;

    let diff = repo.diff_tree_to_workdir(Option::Some(&tree), Option::None)?;

    diff.foreach(
        &mut |_dd, _x| true,
        Option::None,
        Option::None,
        Option::Some(&mut |dd, _dh, dl| {
            if let Ok(line) = str::from_utf8(dl.content()) {
                if is_todo_test(line) {
                    results.push(TodoLine::new(line.into(), str::from_utf8(dd.new_file().path_bytes().unwrap()).unwrap().into(), dl.new_lineno().unwrap()))
                }
                true
            } else {
                false
            }
        })
    )?;
    Ok(())
}

fn extract_for_new_files<F: Fn(&str) -> bool>(repo: &Repository, results: &mut Vec<TodoLine>, is_todo_test: F) -> Result<(), Error> {
    let new_files: Vec<String> = repo.statuses(Option::Some(StatusOptions::new().include_untracked(true)))?.iter()
        .filter(|s| s.status().is_wt_new()) // Only get new files
        .map(|s| s.path().ok_or(failure::err_msg("Invalid utf8 path")).map(Into::into))
        .collect::<Result<Vec<String>, Error>>()?;

    for filename in new_files.iter() {
        BufReader::new(File::open(Path::new(filename))?)
            .lines()
            .filter_map(Result::ok)
            .enumerate()
            .filter(|(_, line)| is_todo_test(line))
            .for_each(|(line_num, line)| results.push(TodoLine::new(line.into(), filename.into(), line_num as u32)));
    }

    Result::Ok(())
}
