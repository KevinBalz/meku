use std::fs;
use std::env;
use std::path::Path;

fn main() {
    for dir in env::args().skip(1){
        map_dir(dir,|path| println!("{:?}",path));
    }
}

fn map_dir<P: AsRef<Path>,F: Fn(&Path)>(dirref: P,func: F) {
    let dir = dirref.as_ref();
    for entry in fs::read_dir(dir).ok().unwrap() {
        let dir = entry.ok().unwrap();
        func(&dir.path());
    }
}
