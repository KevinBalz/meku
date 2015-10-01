extern crate clap;
extern crate meku;

use std::fs;
use std::env;
use std::path;
use std::path::Path;
use clap::{Arg, App};

fn main() {
    let args = App::new("myapp")
        .author("Kevin Balz")
        .version("not nearly finished")
        .about("An awesome build system for making things.")
        .arg(Arg::with_name("source")
            .short("C")
            .long("directory")
            .takes_value(true)
            .help("Sets the source directory (working directory by default)")
        )
        .arg(Arg::with_name("output")
            .index(1)
            .required(true)
            .multiple(true)
            .help("Sets the output folder/s")
        )
        .get_matches();

    let working_dir = env::current_dir().unwrap();
    let src_dir = args.value_of("source").map(Path::new).unwrap_or(working_dir.as_path());

    meku::run_build(src_dir,&args.values_of("output").unwrap());
}
