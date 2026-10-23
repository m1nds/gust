use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

pub const GUST_DIR: &str = ".git";

pub fn init_cmd(args: &[String]) {
    let mut def_path = String::new();
    def_path.push_str(".");

    let out = match args.len() {
        0 => init(Path::new(&def_path)),
        1 => init(Path::new(args.get(0).unwrap_or(&def_path))),
        _ => panic!("Failed to initialize Gust repository: invalid args!"),
    };

    return match out {
        Ok(r) => r,
        Err(error) => panic!("Failed to initialize Gust repository: {error:?}"),
    };
}

pub fn init(path: &Path) -> io::Result<()> {
    if !path.exists() {
        let path_result = fs::create_dir(path);
        match path_result {
            Ok(r) => r,
            Err(error) => return Err(error),
        }
    }

    let s = path.join(GUST_DIR);
    if !path.exists() {
        println!("Git repository already exists in {}!", s.display());
        return Ok(());
    }

    let gust_result = fs::create_dir(&s);
    match gust_result {
        Ok(r) => r,
        Err(error) => return Err(error),
    }

    for dir in ["objects", "refs", "refs/heads", "refs/remotes"] {
        let dir_path = Path::new(&s).join(dir);
        let dir_result = fs::create_dir(&dir_path);
        match dir_result {
            Ok(r) => r,
            Err(error) => return Err(error),
        }
    }

    let mut head = File::create(Path::new(&s).join("HEAD"))?;
    head.write_all(b"ref: refs/heads/master\n")?;

    println!("Initialized empty Gust repository in {}", s.display());

    return Ok(());
}
