extern crate clap;
extern crate meku;

use std::fs;
use std::env;
use std::path;
use std::path::Path;
use clap::{Arg, App, SubCommand};

fn main() {
    let args = App::new("myapp")
        .author("Kevin Balz")
        .version("not nearly finished")
        .about("An awesome build system for making things.")
        .arg_required_else_help(true)
        .arg(Arg::with_name("source")
            .short("C")
            .long("directory")
            .takes_value(true)
            .help("Sets the source directory (working directory by default)")
        )
        .subcommand(SubCommand::with_name("build")
            .arg(Arg::with_name("output")
                .index(1)
                .required(true)
                .multiple(true)
                .help("Sets the output folder/s")
            )
        )
        .subcommand(SubCommand::with_name("run")
            .arg(Arg::with_name("cmd")
                .index(1)
                .required(true)
                .help("The command to run")
            )
            .arg(Arg::with_name("params")
                .index(2)
                .multiple(true)
                .help("The arguments of the command to run")
            )
        )
        .get_matches();

    let working_dir = env::current_dir().unwrap();
    let src_dir = args.value_of("source").map(Path::new).unwrap_or(working_dir.as_path());

    match args.subcommand() {
        ("build",Some(args)) => {
            meku::buildcmd(src_dir,&args.values_of("output").unwrap());
        },
        ("run",Some(args)) => {
            meku::runcmd(src_dir,args.value_of("cmd").unwrap(),&args.values_of("params").unwrap_or(Vec::new()));
        },
        _ => ()
    };


}
