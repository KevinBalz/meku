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

    map_contents(src_dir,&|path| println!("{:?}",path));
    for tar_dir in args.values_of("output").unwrap().iter() {
        println!("OUTPUT FOLDER: {}",tar_dir);
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
