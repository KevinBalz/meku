extern crate tempdir;

use tempdir::TempDir;

use std::fs;
use std::path;
use std::path::Path;

pub fn run_build<S: AsRef<Path>,T: AsRef<Path>>(src_dir: S,target_dirs: &[T]) {
    let files: Vec<_> = iter_contents(&src_dir).map(|f| relative_from(&src_dir,f)).collect();

    let tmp_dir = TempDir::new("meku").unwrap();
    copy_dir_with_filelist(&src_dir,tmp_dir.path(),&files);

    for tar_dir in target_dirs.iter() {
        copy_dir(tmp_dir.path(),tar_dir.as_ref());
    }

}

//Helper Functions

fn copy_dir<S: AsRef<Path>,T: AsRef<Path>>(src_dir: S,tar_dir: T) {
    let files: Vec<_> = iter_contents(&src_dir).map(|f| relative_from(&src_dir,f)).collect();
    copy_dir_with_filelist(src_dir,tar_dir,&files);
}

fn copy_dir_with_filelist<S: AsRef<Path>,T: AsRef<Path>,F: AsRef<Path>>(src_dir: S,tar_dir: T,files: &[F]) {
    println!("OUTPUT FOLDER: {:?}",tar_dir.as_ref());
    for file_rel in files.iter() {
        let src_file = src_dir.as_ref().join(file_rel);
        let tar_file = tar_dir.as_ref().join(file_rel);

        println!("Copy: {:?} to {:?}",&src_file,&tar_file);
        fs::create_dir_all(tar_file.parent().unwrap()).unwrap();
        fs::File::create(&tar_file).unwrap();
        fs::copy(&src_file,&tar_file).unwrap();
    }
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

/// Replacement for Path.relative_from
fn relative_from<S: AsRef<Path>,T: AsRef<Path>>(source: S,path: T) -> path::PathBuf {
    let srciter = source.as_ref().components();
    let mut pathiter = path.as_ref().components();
    for _ in srciter {
        pathiter.next();
    }
    pathiter.as_path().to_path_buf()
}
