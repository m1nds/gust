use std::env;
use std::path::Path;
use builtin::init::GUST_DIR;

mod builtin;
mod utils;

fn usage() {
    println!("gust");
    println!("- init: init a gust repository");
    println!("- hash-object: plumbing tool to hash a Git object and possibly write it");
    println!("- cat-file: plumbing tool to print git objects using hash");
    println!("- add: add file contents to the index");
    println!("- write-tree: write trees from index");
    println!("- commit-tree: build a commit object from a tree");
    println!("- commit: record the index as a new commit on the current branch");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        usage();
        return;
    }

    let cmd = &args[1];
    let cmd_args = &args[2..];

    match cmd.as_str() {
        "init" => {
            builtin::init::init_cmd(cmd_args);
            return;
        }
        "-h" | "--help" => {
            usage();
            return;
        }
        _ => {}
    }

    if !Path::new(GUST_DIR).exists() {
        panic!("fatal: not a gust repository (or any of the parent directories): {}", GUST_DIR);
    }

    match cmd.as_str() {
        "hash-object" => builtin::hash_object::hash_object_cmd(cmd_args),
        "cat-file" => builtin::cat_file::cat_file_cmd(cmd_args),
        "add" => builtin::add::add_cmd(cmd_args),
        "write-tree" => builtin::write_tree::write_tree_cmd(cmd_args),
        "commit-tree" => builtin::commit_tree::commit_tree_cmd(cmd_args),
        "commit" => builtin::commit::commit_cmd(cmd_args),
        _ => {
            usage();
        }
    }
}
