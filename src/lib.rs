extern crate tempdir;
extern crate yaml_rust;
extern crate regex;

use tempdir::TempDir;
use yaml_rust::{YamlLoader,Yaml};
use regex::Regex;

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
            if pattern_match(ext,&file) {
                cmds_raw.push( (file,action) );
                continue 'filel;
            }
        }
        filescpy.push(relative_from(&src_dir,file));
    }

    clone_directory_structure(&src_dir,&tar_dir);

    // Process all found files
    for (file,action) in cmds_raw {
        match action {
            &Action::Command(ref cmds) => for cmdstr in cmds {
                let mut cmditer = argenize(cmdstr);
                match cmditer.next().unwrap() {
                    "mv" => {
                        let src = apply_holder(&file, &src_dir, &tar_dir, cmditer.next().unwrap());
                        let tar = apply_holder(&file, &src_dir, &tar_dir, cmditer.next().unwrap());
                        println!("moving {} to {}",&src,&tar );
                        fs::copy(&src,&tar).unwrap();
                        fs::remove_file(&src).unwrap();
                    },
                    cmdst => {
                        let mut cmd = Command::new(cmdst);
                        for arg in cmditer {
                            cmd.arg (apply_holder(&file, &src_dir, &tar_dir, arg));
                        }
                        println!("Executing: {:?}",cmd);
                        cmd.status().unwrap();
                    }
                }
            }
        }
    }

    // Copy all normal files
    copy_dir_with_filelist(&src_dir,&tar_dir,&filescpy);
}

fn apply_holder<S: AsRef<Path>,SD: AsRef<Path>,T: AsRef<Path>>(src_file: S,src_dir: SD,tar_dir: T,arg: &str) -> String {
    arg
        //TODO: separate into another function
        .replace("%{src_file}",src_file.as_ref().to_str().unwrap())
        .replace("%{tar_dir}", tar_dir.as_ref().to_str().unwrap())
        .replace("%{src_file_stem}",src_file.as_ref().file_stem().unwrap().to_str().unwrap())
        .replace("%{src_file_noext}",src_file.as_ref().with_extension("").to_str().unwrap())
        .replace("%{src_rel_noext}",relative_from(&src_dir,&src_file).with_extension("").to_str().unwrap())
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

#[test]
fn test_pattern_match_without_stars() {
    assert_eq!(true,pattern_match("main.moon","main.moon"));
    assert_eq!(false,pattern_match("main.moon","main.c"));
}

//TODO: expand tests
#[test]
fn test_pattern_match_single_star() {
    //Single star
    assert_eq!(true,pattern_match("*.moon","main.moon"));
    assert_eq!(false,pattern_match("*.moon","main.c"));
    assert_eq!(false,pattern_match("*.moon","a.moon.c"));
}

#[test]
fn test_pattern_match_single_star_with_previous_folders() {
    //Single star with previous folders
    assert_eq!(false,pattern_match("*.moon","src/main.moon"));
    assert_eq!(false,pattern_match("*.moon","src/main.c"));
}

#[test]
fn test_pattern_match_double_star() {
    //Double star
    assert_eq!(true,pattern_match("**.moon","src/main.moon"));
    assert_eq!(false,pattern_match("**.moon","src/main.moon.c"));
    assert_eq!(true,pattern_match("**.moon","main.moon"));
    assert_eq!(false,pattern_match("**.moon","src/main.c"));
}

/// Checks if two file patterns with "wildcards" match
//TODO: DO the real thing instead of checking the filending
//TODO: Return value to determine how specific it matched (for deciding which pattern of more matches most)
fn pattern_match<P: AsRef<Path>>(pattern: &str,path: P) -> bool {
    //Convert pattern to regex
    let re = Regex::new("[^\\\\/]*\\.moon").unwrap();

    assert_eq!(true, re.is_match("test.moon"));
    assert_eq!(false, re.is_match("test.c"));


    //TODO: do above

    path.as_ref().extension().unwrap() == Path::new(pattern).extension().unwrap()
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

fn clone_directory_structure<S: AsRef<Path>,T: AsRef<Path>>(src_dir: S,tar_dir: T) {
    for dir in iter_dirs(&src_dir).map(|f| tar_dir.as_ref().join(relative_from(&src_dir,f)) ) {
        std::fs::create_dir_all(dir).unwrap();
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

struct IterDirs {
    diriter: IterDir,
    recur:   Option<Box<IterDirs>>
}

impl Iterator for IterDirs {
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
                    self.next()
                }
                else if ftype.is_dir() {
                    self.recur = Some(Box::new(iter_dirs(&path)));
                    Some(path)
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

fn iter_dirs<P: AsRef<Path>>(dirref: P) -> IterDirs {
    IterDirs {diriter: iter_dir(dirref),recur: None }
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

#[test]
fn test_argenize() {
    assert_eq!(vec!("a","b","c"),argenize("a b  c").collect::<Vec<&str>>());
    assert_eq!(vec!("a","b c","d"),argenize("a \"b c\"  d").collect::<Vec<&str>>());
}

/// Splits the string into a slice array for process::Command
/// Takes care of "...  ... " 's
fn argenize<'a>(argstr: &'a str) -> Argenize<'a> {
    Argenize {argstr: argstr,index: argstr.char_indices()}
}

struct Argenize<'a> {
    argstr: &'a str,
    index: std::str::CharIndices<'a>
}

impl<'a> Iterator for  Argenize<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let mut start;
        let mut ch;
        //Scan to first non whitespace
        loop {
            match self.index.next() {
                Some( (i,c) ) => {
                    start = i;
                    ch = c;
                },
                None => return None // End of string
            }
            if ch != ' ' {
                break;
            }
        }

        let has_qoute = ch == '"';
        if has_qoute {
            start +=1; // skip the trailing qoute
        }

        let mut ind = start;
        loop {
            match self.index.next() {
                Some( (i,c) ) => {
                    if !has_qoute && c == ' ' {
                        return Some(&self.argstr[start .. ind+1])
                    }
                    else if  has_qoute && c == '"' {
                        return Some(&self.argstr[start .. ind+1])
                    }
                    else {
                        ind = i;
                    }
                },
                None => {
                    if has_qoute {
                        return None //End of string without closing qoute
                    }
                    else {
                        return Some(&self.argstr[start .. ind+1])
                    }
                }
            }
        }
    }
}
