use crate::builtin::init::GUST_DIR;
use crate::utils::object::Object;
use std::io::{self, Error, ErrorKind};
use std::path::Path;

pub fn cat_file_cmd(args: &[String]) {
    let hash = match args.len() {
        1 => args.get(0).unwrap(),
        _ => panic!("Failed to retrieve hash!"),
    };

    return match cat_file(&hash) {
        Ok(_) => (),
        Err(error) => panic!("Failed to initialize Gust repository: {error:?}"),
    };
}

pub fn cat_file(hash: &String) -> io::Result<()> {
    if hash.len() != 64 {
        return Err(Error::new(ErrorKind::Other, "Invalid hash!"));
    }

    let path = Path::new(GUST_DIR)
        .join("objects")
        .join(&hash[..2])
        .join(&hash[2..]);

    let obj = match Object::cat_file(&path) {
        Ok(o) => o,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to parse object: {e}"),
            ))
        }
    };

    match obj {
        Object::Blob(blob) => println!("{}", String::from_utf8_lossy(&blob.data)),
        _ => (),
    };

    return Ok(());
}
