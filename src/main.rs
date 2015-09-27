use std::fs;
use std::env;
use std::path::Path;

fn main() {
    for dir in env::args().skip(1) {
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
