extern crate tempdir;
extern crate yaml_rust;

use tempdir::TempDir;
use yaml_rust::{YamlLoader,Yaml};

use std::fs;
use std::path;
use std::path::Path;
use std::io::Read;
use std::process::Command;

#[derive(Debug,PartialEq)]
enum Action {
    Command(Vec<String>)
}

#[derive(Debug,PartialEq)]
struct Rule {
    pattern: String,
    action: Action
}

#[test]
fn test_parse_rule_command() {
    let testcmd = "moonc file1 file2";
    let mut map = std::collections::BTreeMap::new();
    map.insert(Yaml::String("commands".to_string()), Yaml::Array(vec!( Yaml::String(testcmd.to_string()) )));

    let key = Yaml::String("*.moon".to_string());
    let value = Yaml::Hash(map);

    let expected = Rule {
        pattern: "*.moon".to_string(),
        action: Action::Command(vec!(testcmd.to_string() ))
    };
    assert_eq!(expected,parse_rule(&key,&value));
}

fn parse_rule(pattern: &Yaml,args: &Yaml) -> Rule {
    let pattern = pattern.as_str().unwrap().to_string();

    let mut cmds: Vec<_> = Vec::new();
    for cmd in args["commands"].as_vec().unwrap().iter() {
        cmds.push(cmd.as_str().unwrap().to_string());
    }

    Rule { pattern: pattern.to_string(),action: Action::Command(cmds) }
}

/// Executes the meku build process for `src_dir` and write the results to `tar_dir`
pub fn build<S: AsRef<Path>,T: AsRef<Path>>(src_dir: S,tar_dir: T) {
    let (mekufiles,files): (Vec<_>, Vec<_>) = iter_contents(&src_dir).partition(|f| match f.file_name() {
        Some(name) => name == "meku.yml",
        None       => false
    });

    // Parse all meku.yml files
    let mut rules: Vec<_> = Vec::new();
    for meku in mekufiles.iter() {
        let mut file = fs::File::open(meku).unwrap();
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap();
        let yaml = YamlLoader::load_from_str(&string).unwrap();

        for (key,value) in yaml[0].as_hash().unwrap().iter() {
            let rule = parse_rule(key,value);

            rules.push(rule);
        }
    }

    // Find all files which have to be processed
    let mut cmds_raw = Vec::new();
    let mut filescpy: Vec<_> = Vec::new();
    'filel: for file in files {
        for rule in rules.iter() {
            let ext = &rule.pattern;
            let action = &rule.action;
            if file.extension().unwrap() == ext.as_os_str() {
                cmds_raw.push( (file,action) );
                continue 'filel;
            }
        }
        filescpy.push(relative_from(&src_dir,file));
    }

    // Process all found files
    for (file,action) in cmds_raw {
        match action {
            &Action::Command(ref cmds) => for cmdstr in cmds {
                let mut cmditer = cmdstr.split_whitespace();
                let mut cmd = Command::new(cmditer.next().unwrap());
                for arg in cmditer {
                    let newarg = arg
                        .replace("%{src_file}",file.to_str().unwrap())
                        .replace("%{tar_dir}", tar_dir.as_ref().to_str().unwrap())
                        .replace("%{src_file_stem}",file.file_stem().unwrap().to_str().unwrap());
                    cmd.arg (newarg);
                }
                println!("Executing: {:?}",cmd);
                cmd.status().unwrap();
            }
        }
    }

    // Copy all normal files
    copy_dir_with_filelist(&src_dir,&tar_dir,&filescpy);
}

/// Does the same as executing the executable with `meku build <target_dirs>...`
pub fn buildcmd<S: AsRef<Path>,T: AsRef<Path>>(src_dir: S,target_dirs: &[T]) {
    let tmp_dir = TempDir::new("meku-build").unwrap();
    build(&src_dir,tmp_dir.path());

    for tar_dir in target_dirs.iter() {
        copy_dir(tmp_dir.path(),tar_dir.as_ref());
    }

}

/// Does the same as executing the executable with `meku run <cmdname> <params>...`
pub fn runcmd<S: AsRef<Path>>(src_dir: S,cmdname: &str,params: &[&str]) {
    let tmp_dir = TempDir::new("meku-run").unwrap();
    build(&src_dir,tmp_dir.path());

    let mut cmd = Command::new(cmdname);
    for param in params.iter()
        .map(|p|
            if *p == "%{}"
                {tmp_dir.path().to_str().unwrap()}
            else {
                *p
            }) {
        cmd.arg(param);
    }
    println!("Executing {:?}",cmd);
    cmd.status().unwrap();
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

//HACK Replacement for Path.relative_from which is unstable
fn relative_from<S: AsRef<Path>,T: AsRef<Path>>(source: S,path: T) -> path::PathBuf {
    let srciter = source.as_ref().components();
    let mut pathiter = path.as_ref().components();
    for _ in srciter {
        pathiter.next();
    }
    pathiter.as_path().to_path_buf()
}
