use crate::utils::object::Object;
use std::fs::File;
use std::io::{self, Error, ErrorKind, Read};
use std::path::Path;

pub fn hash_object_cmd(args: &[String]) {
    let empty_str = String::new();

    let mut def_obj_type = String::new();
    def_obj_type.push_str("blob");

    let obj_type = args.get(0).unwrap_or(&def_obj_type);
    let path = Path::new(args.get(1).unwrap_or(&empty_str));

    let write = match args.get(2).unwrap_or(&empty_str).as_str() {
        "--write" => true,
        _ => false,
    };

    match hash_object(obj_type, path, write) {
        Ok(_) => (),
        Err(error) => panic!("Failed to initialize Gust repository: {error:?}"),
    }
}

pub fn hash_object(obj_type: &String, path: &Path, write: bool) -> io::Result<()> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other,format!("Failed to open file: {e}")))
        }
    };

    let mut data = Vec::new();
    match file.read_to_end(&mut data) {
        Ok(_) => (),
        Err(e) => {
            return Err(Error::new(ErrorKind::Other,format!("Failed to read file: {e}")))
        }
    }

    let obj = match Object::read_raw_object(obj_type, &data) {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    let hash = match obj.hash_object(write) {
        Ok(h) => h,
        Err(e) => return Err(e),
    };

    println!("{}", hash);

    return Ok(());
}
