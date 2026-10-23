use std::fs::create_dir;
use std::fs::File;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use crate::builtin::init::GUST_DIR;

use sha2::{Digest, Sha256};
use zlib_rs::{
    compress_bound, compress_slice, decompress_slice, DeflateConfig, InflateConfig, ReturnCode,
};

const HASH_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub enum Object {
    Blob(BlobObj),
    Tree(TreeObj),
    Commit(CommitObj),
}

#[derive(Debug, Clone)]
pub struct BlobObj {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: [u8; HASH_SIZE],
}

#[derive(Debug, Clone)]
pub struct TreeObj {
    pub entries: Vec<TreeEntry>,
}

impl TreeObj {
    pub fn dump_entries(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for elt in &self.entries {
            data.extend_from_slice(elt.mode.as_bytes());
            data.push(b' ');
            data.extend_from_slice(elt.name.as_bytes());
            data.push(b'\x00');
            data.extend_from_slice(&elt.hash);
        }
        return data;
    }
}

#[derive(Debug, Clone)]
pub struct CommitObj {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
}

impl CommitObj {
    pub fn dump_commit(&self) -> Vec<u8> {
        let mut data = format!("tree {}\n", self.tree);

        for parent in &self.parents {
            data.push_str(&format!("parent {}\n", parent));
        }

        data.push_str(&format!("author {}\n", self.author));
        data.push_str(&format!("committer {}\n", self.committer));
        data.push_str(&format!("\n{}\n", self.message));

        return data.into_bytes();
    }
}

impl Object {
    fn write_object(hash: &String, data: &[u8]) -> Result<(), Error> {
        let path = Path::new(GUST_DIR).join("objects").join(&hash[..2]);
        match create_dir(&path) {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to create directory: {}", e),
                ))
            }
        }

        let file_path = path.join(&hash[2..]);

        match File::create(file_path) {
            Ok(mut file) => match file.write_all(data) {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Failed to write object: {}", e),
                    ))
                }
            },
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to create object file: {}", e),
                ))
            }
        }
    }

    pub fn read_raw_object(obj_type: &str, data: &[u8]) -> io::Result<Object> {
        match obj_type {
            "blob" => Ok(Object::Blob(BlobObj {
                data: data.to_vec(),
            })),
            "tree" => {
                let mut entries = Vec::new();
                let mut idx = 0;
                while idx < data.len() {
                    let space_pos = idx + data[idx..].iter().position(|&b| b == b' ').unwrap();
                    let null_pos = space_pos + 1 + data[space_pos + 1..].iter().position(|&b| b == b'\x00').unwrap();

                    let mode_bytes = &data[idx..space_pos];
                    let path_bytes = &data[space_pos + 1..null_pos];
                    let hash_bytes = &data[null_pos + 1..null_pos + 1 + HASH_SIZE];
                    
                    let mode = std::str::from_utf8(mode_bytes).unwrap().to_string();
                    let name = std::str::from_utf8(path_bytes).unwrap().to_string();
                    
                    let mut hash = [0u8; HASH_SIZE];
                    hash.copy_from_slice(hash_bytes);

                    entries.push(TreeEntry {
                        mode,
                        name,
                        hash,
                    });

                    idx = null_pos + 1 + HASH_SIZE;
                }

                return Ok(Object::Tree(TreeObj {
                    entries: entries,
                }))},
            "commit" => {
                let content = std::str::from_utf8(data).map_err(|e| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Failed to parse raw commit as UTF-8: {}", e),
                    )
                })?;

                let mut parts = content.splitn(2, "\n\n");
                let header = parts.next().unwrap_or("");
                let message = parts.next().unwrap_or("");

                let mut tree = String::new();
                let mut parents = Vec::new();
                let mut author = String::new();
                let mut committer = String::new();

                for line in header.lines() {
                    let mut splitter = line.splitn(2, ' ');
                    let key = splitter.next().unwrap_or("");
                    let val = splitter.next().unwrap_or("");
                    match key {
                        "tree" => tree = val.to_string(),
                        "parent" => parents.push(val.to_string()),
                        "author" => author = val.to_string(),
                        "committer" => committer = val.to_string(),
                        _ => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                format!("Failed to process commit field: {}", key),
                            ))
                        }
                    }
                }

                Ok(Object::Commit(CommitObj {
                    tree,
                    parents,
                    author,
                    committer,
                    message: message.to_string(),
                }))
            }
            _ => Err(Error::new(
                ErrorKind::Other,
                format!("Unsupported raw object type: {}", obj_type),
            )),
        }
    }

    pub fn hash_object(&self, write: bool) -> io::Result<String> {
        let mut data = Vec::new();
        match self {
            Object::Blob(blob) => {
                data.extend_from_slice(&blob.data);
            }
            Object::Tree(tree) => {
                data.extend_from_slice(&tree.dump_entries());
            }
            Object::Commit(commit) => {
                data.extend_from_slice(&commit.dump_commit());
            }
        }

        let data_len = data.len();
        let obj_type = match self {
            Object::Blob(_) => "blob",
            Object::Tree(_) => "tree",
            Object::Commit(_) => "commit",
        };

        let header = format!("{} {}\0", obj_type, data_len);

        let mut complete = Vec::new();
        complete.extend_from_slice(header.as_bytes());
        complete.extend_from_slice(&data);

        let mut sha256 = Sha256::new();
        sha256.update(&complete);
        let hash_bytes: [u8; 32] = sha256.finalize().into();
        let hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        if write {
            let mut compressed_bytes = vec![0u8; compress_bound(complete.len())];
            let (compressed, rc) = compress_slice(
                &mut compressed_bytes,
                complete.as_slice(),
                DeflateConfig::default(),
            );
            if rc != ReturnCode::Ok {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to compress: {:?}", rc),
                ));
            }
            match Object::write_object(&hash, &compressed) {
                Ok(_) => (),
                Err(error) => panic!("Failed write_object(): {error:?}"),
            }
        }

        return Ok(hash);
    }

    pub fn cat_file(path: &Path) -> io::Result<Object> {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to open file: {}", e),
                ))
            }
        };

        let mut compressed = Vec::new();
        match file.read_to_end(&mut compressed) {
            Ok(_) => (),
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to read file: {}", e),
                ))
            }
        };

        let mut decompressed_bytes = vec![0u8; compressed.len() * 4];
        let (decompressed, rc) = decompress_slice(
            &mut decompressed_bytes,
            &compressed,
            InflateConfig::default(),
        );

        if rc != ReturnCode::Ok {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to decompress: {:?}", rc),
            ));
        }

        let null_byte = decompressed
            .iter()
            .position(|&b| b == b'\x00')
            .expect("Invalid object: missing object null byte");
        let header = &decompressed[..null_byte];
        let obj_data = &decompressed[null_byte + 1..];

        let header_space = header
            .iter()
            .position(|&b| b == b' ')
            .expect("Invalid object: missing header space byte");
        let obj_type = &header[..header_space];
        let size_str = match str::from_utf8(&header[header_space + 1..]) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse object size: {}", e),
                ))
            }
        };

        let size = size_str.parse::<usize>().unwrap();
        assert_eq!(size, obj_data.len());

        return match str::from_utf8(obj_type) {
            Ok(s) => Object::read_raw_object(s, obj_data),
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to process object type matching!"),
                ))
            }
        };
    }
}
