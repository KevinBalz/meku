extern crate clap;

use std::fs;
use std::env;
use std::path::Path;
use clap::{Arg, App};

fn main() {
    let args = App::new("myapp")
        .author("Kevin Balz")
        .version("not nearly finished")
        .about("An awesome build system for making things.")
        .arg(Arg::with_name("output")
            .index(1)
            .required(true)
            .multiple(true)
            .help("Sets the output folder/s")
        )
        .get_matches();
    for dir in args.values_of("output").unwrap().iter() {
        map_contents(dir,&|path| println!("{:?}",path));
    }
}

fn map_contents<P: AsRef<Path>,F: Fn(&Path)>(dirref: P,func: &F) {
    map_dir(dirref,|path,ftype|
        if ftype.is_file() {
            func(path);
        }
        else if ftype.is_dir() {
            map_contents(path,func);
        }
        else if ftype.is_symlink() {
            panic!("Symlink in map_contents!");
        }
        else {
            panic!("Unknown File Type");
        }
    )
}

fn map_dir<P: AsRef<Path>,F: Fn(&Path,fs::FileType)>(dirref: P,func: F) {
    let dir = dirref.as_ref();
    for entry in fs::read_dir(dir).ok().unwrap() {
        let dir = entry.ok().unwrap();
        func(&dir.path(),dir.file_type().unwrap());
    }
}
