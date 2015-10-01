use std::fs;
use std::path;
use std::path::Path;

pub fn run_build<S: AsRef<Path>,T: AsRef<Path>>(src_dir: S,target_dirs: &[T]) {
    map_contents(src_dir,|path| println!("{:?}",path));
    for tar_dir in target_dirs.iter() {
        println!("OUTPUT FOLDER: {:?}",tar_dir.as_ref());
    }

}

//Helper Functions

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
