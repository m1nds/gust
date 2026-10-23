use crate::builtin::init::GUST_DIR;
use crate::utils::index::Index;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::Path;

pub fn add_cmd(args: &[String]) {
    if args.is_empty() {
        panic!("Failed to retrieve paths!");
    }

    let mut index = match Index::read_index() {
        Ok(idx) => idx,
        Err(error) => panic!("Failed to read index: {error:?}"),
    };

    for path_str in args {
        let path = Path::new(path_str);
        if let Err(error) = add_recursive(path, &mut index) {
            panic!("Failed to add path '{}': {:?}", path_str, error);
        }
    }

    if let Err(error) = index.write_index() {
        panic!("Failed to write index: {error:?}");
    }
}

pub fn add_recursive(path: &Path, index: &mut Index) -> io::Result<()> {
    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, format!("Path does not exist: {:?}", path)));
    }

    if let Some(name) = path.file_name() {
        if name == GUST_DIR {
            return Ok(());
        }
    }

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            add_recursive(&entry.path(), index)?;
        }
    } else if path.is_file() {
        index.add_file(path)?;
    }

    Ok(())
}
