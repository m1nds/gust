use crate::utils::index::{Index, IndexEntry};
use crate::utils::object::{Object, TreeEntry, TreeObj, HASH_SIZE};
use std::io::{self, Error, ErrorKind};

pub fn write_tree_cmd(args: &[String]) {
    if !args.is_empty() {
        panic!("write-tree takes no arguments!");
    }

    match write_tree() {
        Ok(hex_hash) => println!("{}", hex_hash),
        Error => panic!("Failed to write tree"),
    }
}

pub fn write_tree() -> io::Result<String> {
    let index = Index::read_index()?;

    if index.entries.is_empty() {
        return Err(Error::new(ErrorKind::Other, "Cannot write tree: index is empty"));
    }

    let entry_refs: Vec<&IndexEntry> = index.entries.iter().collect();

    let root_hash_bytes = write_tree_recursive(&entry_refs, 0)?;

    let hex_hash: String = root_hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    Ok(hex_hash)
}

fn write_tree_recursive(entries: &[&IndexEntry], depth: usize) -> io::Result<[u8; HASH_SIZE]> {
    let mut tree_entries = Vec::new();
    let mut i = 0;

    while i < entries.len() {
        let parts: Vec<&str> = entries[i].path.split('/').collect();

        match parts.get(depth) {
            Some(&name) if depth + 1 == parts.len() => {
                tree_entries.push(TreeEntry {
                    mode: format!("{:o}", entries[i].mode),
                    name: name.to_string(),
                    hash: entries[i].hash,
                });
                i += 1;
            },
            Some(&name) => {
                let start = i;
                while i < entries.len() {
                    let p: Vec<&str> = entries[i].path.split('/').collect();
                    match p.get(depth) {
                        Some(&p_name) if p_name == name => i += 1,
                        _ => break,
                    }
                }

                let sub_entries = &entries[start..i];
                let sub_hash = match write_tree_recursive(sub_entries, depth + 1) {
                    Ok(hash) => hash,
                    Err(e) => return Err(e),
                };

                tree_entries.push(TreeEntry {
                    mode: "40000".to_string(),
                    name: name.to_string(),
                    hash: sub_hash,
                });
            },
            None => {
                i += 1;
            }
        }
    }

    let tree_obj = Object::Tree(TreeObj { entries: tree_entries });
    let hex_hash = match tree_obj.hash_object(true) {
        Ok(hash) => hash,
        Err(e) => return Err(e),
    };

    let mut hash_bytes = [0u8; HASH_SIZE];
    for (idx, chunk) in hex_hash.as_bytes().chunks(2).enumerate() {
        let s = match std::str::from_utf8(chunk) {
            Ok(str_val) => str_val,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Invalid UTF-8 in hash: {}", e))),
        };

        hash_bytes[idx] = match u8::from_str_radix(s, 16) {
            Ok(val) => val,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Invalid hex hash: {}", e))),
        };
    }

    Ok(hash_bytes)
}
