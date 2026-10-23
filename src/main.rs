use std::env;

mod builtin;
mod utils;

fn usage() {
    println!("gust");
    println!("- init: init a gust repository");
    println!("- hash-object: plumbing tool to hash a Git object and possibly write it");
    println!("- cat-file: plumbing tool to print git objects using hash");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        usage();
        return;
    }

    let cmd = &args[1];
    let args = &args[2..];

    match cmd.as_str() {
        "init" => builtin::init::init_cmd(args),
        "hash-object" => builtin::hash_object::hash_object_cmd(args),
        "cat-file" => builtin::cat_file::cat_file_cmd(args),
        "-h" | "--help" | _ => usage(),
    }
}
