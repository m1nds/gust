use std::env;

mod builtin;

fn usage() {
    println!("gust");
    println!("- init: init a gust repository");
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
        "-h" | "--help" | _ => usage()
    }
}
