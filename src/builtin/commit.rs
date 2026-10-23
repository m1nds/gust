use crate::builtin::commit_tree::{commit_tree, read_commit, read_ref};
use crate::builtin::init::GUST_DIR;
use crate::builtin::write_tree::write_tree;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn commit_cmd(args: &[String]) {
    let empty_str = String::new();

    let flag = args.get(0).unwrap_or(&empty_str);
    let message = args.get(1).unwrap_or(&empty_str);

    if flag != "-m" || message.is_empty() || args.len() > 2 {
        panic!("Failed to retrieve arguments!");
    }

    match commit(message) {
        Ok(output) => println!("{}", output),
        Err(error) => panic!("Failed to commit: {error}"),
    }
}

fn update_ref(ref_path: &str, commit_hash: &str) -> Result<(), String> {
    let full_path = Path::new(GUST_DIR).join(ref_path);

    match full_path.parent() {
        Some(parent) => match fs::create_dir_all(parent) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to create ref directory: {}", e)),
        },
        None => (),
    }

    match File::create(&full_path) {
        Ok(mut file) => match file.write_all(format!("{}\n", commit_hash).as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write ref: {}", e)),
        },
        Err(e) => Err(format!("Failed to create ref file: {}", e)),
    }
}

pub fn commit(message: &str) -> Result<String, String> {
    let tree_hash = match write_tree() {
        Ok(hash) => hash,
        Err(e) => return Err(format!("Failed to write tree: {}", e)),
    };

    let ref_path = match read_ref() {
        Some(path) => path,
        None => return Err("HEAD is detached, refusing to commit".to_string()),
    };

    let is_root = read_commit(&ref_path).is_none();

    let commit_hash = commit_tree(&tree_hash, message)?;

    update_ref(&ref_path, &commit_hash)?;

    let branch = match ref_path.strip_prefix("refs/heads/") {
        Some(name) => name,
        None => &ref_path,
    };

    let short_hash = &commit_hash[..7];
    let root_marker = if is_root { "(root-commit) " } else { "" };

    Ok(format!("[{} {}{}] {}", branch, root_marker, short_hash, message))
}
