extern crate clap;

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

    map_contents(src_dir,|path| println!("{:?}",path));
    for tar_dir in args.values_of("output").unwrap().iter() {
        println!("OUTPUT FOLDER: {}",tar_dir);
    }
}

fn map_contents<P: AsRef<Path>,F: Fn(path::PathBuf) -> R,R>(dirref: P,func: F) -> Vec<R> {
    iter_contents(dirref).map(|p| func(p) ).collect()
}

fn map_dir<P: AsRef<Path>,F: Fn(path::PathBuf,fs::FileType) -> R,R>(dirref: P,func: F) -> Vec<R> {
    iter_dir(dirref).map(|(p,t)| func(p,t) ).collect()
}

struct IterDir {
    diriter: fs::ReadDir
}

impl Iterator for IterDir {
    type Item = (path::PathBuf,fs::FileType);

    fn next(&mut self) -> Option<(path::PathBuf,fs::FileType)> {
        self.diriter.next().map(|entry| {
            let dir = entry.ok().unwrap();
            (dir.path(),dir.file_type().unwrap())
        })
    }
}


fn iter_dir<P: AsRef<Path>>(dirref: P) -> IterDir {
    let dir = dirref.as_ref();
    IterDir {diriter: fs::read_dir(dir).unwrap() }
}

struct IterContents {
    diriter: IterDir,
    recur:   Option<Box<IterContents>>
}

impl Iterator for IterContents {
    type Item = path::PathBuf;

    fn next(&mut self) -> Option<path::PathBuf> {
        match self.recur {
            Some(ref mut iter) => match iter.next() {
                    Some(v) => return Some(v),
                    None    => ()
                },
            None => ()
        };
        self.recur = None;
        match self.diriter.next() {
            Some((path,ftype)) =>
                if ftype.is_file() {
                    Some(path)
                }
                else if ftype.is_dir() {
                    self.recur = Some(Box::new(iter_contents(path)));
                    self.next()
                }
                else if ftype.is_symlink() {
                    panic!("Symlink in iter_contents!");
                }
                else {
                    panic!("Unknown File Type");
                },
            None => None
        }
    }
}

fn iter_contents<P: AsRef<Path>>(dirref: P) -> IterContents {
    IterContents {diriter: iter_dir(dirref),recur: None }
}
