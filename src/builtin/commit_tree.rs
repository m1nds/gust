use crate::builtin::init::GUST_DIR;
use crate::utils::config::GitConfig;
use crate::utils::object::{CommitObj, Object};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn commit_tree_cmd(args: &[String]) {
    let empty_str = String::new();

    let tree_hash = args.get(0).unwrap_or(&empty_str);
    let flag = args.get(1).unwrap_or(&empty_str);
    let message = args.get(2).unwrap_or(&empty_str);

    if tree_hash.is_empty() || flag != "-m" || message.is_empty() {
        panic!("Failed to retrieve arguments!");
    }

    match commit_tree(tree_hash, message) {
        Ok(hash) => println!("{}", hash),
        Err(error) => panic!("Failed to commit-tree: {error}"),
    }
}

pub fn read_ref() -> Option<String> {
    let head_path = Path::new(GUST_DIR).join("HEAD");
    let file_content = match File::open(&head_path) {
        Ok(mut file) => {
            let mut data = Vec::new();
            match file.read_to_end(&mut data) {
                Ok(_) => match String::from_utf8(data) {
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }
        Err(_) => None,
    };

    let content = match file_content {
        Some(c) => c,
        None => return None,
    };

    let trimmed = content.trim();
    if !trimmed.starts_with("ref: ") {
        return None;
    }

    Some(trimmed[5..].to_string())
}

pub fn read_commit(ref_path: &str) -> Option<String> {
    let full_path = Path::new(GUST_DIR).join(ref_path);

    match File::open(&full_path) {
        Ok(mut file) => {
            let mut commit_id = String::new();
            match file.read_to_string(&mut commit_id) {
                Ok(_) => {
                    let clean_id = commit_id.trim();
                    if !clean_id.is_empty() {
                        Some(clean_id.to_string())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

pub fn commit_tree(tree_hash: &str, message: &str) -> Result<String, String> {
    let mut parents = Vec::new();

    match read_ref() {
        Some(ref_path) => {
            match read_commit(&ref_path) {
                Some(commit_hash) => {
                    parents.push(commit_hash);
                }
                None => {}
            }
        }
        None => {}
    }

    let config = GitConfig::get();

    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => 0,
    };

    let author_identity = format!("{} <{}> {} +0000", config.user.name, config.user.email, timestamp);
    let committer_identity = author_identity.clone();
    let commit_obj = Object::Commit(CommitObj {
        tree: tree_hash.to_string(),
        parents,
        author: author_identity,
        committer: committer_identity,
        message: message.to_string(),
    });

    match commit_obj.hash_object(true) {
        Ok(hash) => Ok(hash),
        Err(e) => Err(format!("Failed to write commit object: {}", e)),
    }
}
