use std::fs::File;
use std::io::{self, Error, ErrorKind, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use sha2::{Digest, Sha256};

use crate::builtin::init::GUST_DIR;
use crate::utils::object::{Object, HASH_SIZE};

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub ctime_secs: u32,
    pub ctime_nanos: u32,
    pub mtime_secs: u32,
    pub mtime_nanos: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub file_size: u32,
    pub hash: [u8; HASH_SIZE],
    pub flags: u16,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub entries: Vec<IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Index { entries: Vec::new() }
    }

    pub fn read_index() -> io::Result<Self> {
        let index_path = Path::new(GUST_DIR).join("index");
        if !index_path.exists() {
            return Ok(Index::new());
        }

        let mut file = File::open(&index_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 12 || &data[0..4] != b"DIRC" {
            return Err(Error::new(ErrorKind::Other, "Invalid index file or signature"));
        }

        let entry_count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let mut entries = Vec::with_capacity(entry_count);
        let mut cursor = 12;

        for _ in 0..entry_count {
            let entry_start = cursor;
            if cursor + 62 + HASH_SIZE > data.len() {
                return Err(Error::new(ErrorKind::Other, "Truncated index entry"));
            }

            let read_u32 = |c: &mut usize| -> u32 {
                let val = u32::from_be_bytes([data[*c], data[*c+1], data[*c+2], data[*c+3]]);
                *c += 4;
                val
            };

            let ctime_secs = read_u32(&mut cursor);
            let ctime_nanos = read_u32(&mut cursor);
            let mtime_secs = read_u32(&mut cursor);
            let mtime_nanos = read_u32(&mut cursor);
            let dev = read_u32(&mut cursor);
            let ino = read_u32(&mut cursor);
            let mode = read_u32(&mut cursor);
            let uid = read_u32(&mut cursor);
            let gid = read_u32(&mut cursor);
            let file_size = read_u32(&mut cursor);

            let mut hash = [0u8; HASH_SIZE];
            hash.copy_from_slice(&data[cursor..cursor + HASH_SIZE]);
            cursor += HASH_SIZE;

            let flags = u16::from_be_bytes([data[cursor], data[cursor + 1]]);
            cursor += 2;

            let path_start = cursor;
            let null_offset = data[path_start..]
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| Error::new(ErrorKind::Other, "Missing null byte in index path"))?;
            let null_pos = path_start + null_offset;

            let path = std::str::from_utf8(&data[path_start..null_pos])
                .map_err(|e| Error::new(ErrorKind::Other, format!("Invalid index path: {}", e)))?
                .to_string();

            entries.push(IndexEntry {
                ctime_secs,
                ctime_nanos,
                mtime_secs,
                mtime_nanos,
                dev,
                ino,
                mode,
                uid,
                gid,
                file_size,
                hash,
                flags,
                path,
            });

            let entry_len = (null_pos + 1 - entry_start + 7) & !7;
            cursor = entry_start + entry_len;
        }

        Ok(Index { entries })
    }

    pub fn write_index(&self) -> io::Result<()> {
        let index_path = Path::new(GUST_DIR).join("index");
        let mut data = Vec::new();

        data.extend_from_slice(b"DIRC");
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());

        for entry in &self.entries {
            let entry_start = data.len();

            data.extend_from_slice(&entry.ctime_secs.to_be_bytes());
            data.extend_from_slice(&entry.ctime_nanos.to_be_bytes());
            data.extend_from_slice(&entry.mtime_secs.to_be_bytes());
            data.extend_from_slice(&entry.mtime_nanos.to_be_bytes());
            data.extend_from_slice(&entry.dev.to_be_bytes());
            data.extend_from_slice(&entry.ino.to_be_bytes());
            data.extend_from_slice(&entry.mode.to_be_bytes());
            data.extend_from_slice(&entry.uid.to_be_bytes());
            data.extend_from_slice(&entry.gid.to_be_bytes());
            data.extend_from_slice(&entry.file_size.to_be_bytes());
            data.extend_from_slice(&entry.hash);
            data.extend_from_slice(&entry.flags.to_be_bytes());
            data.extend_from_slice(entry.path.as_bytes());
            data.push(b'\x00');

            while (data.len() - entry_start) % 8 != 0 {
                data.push(b'\x00');
            }
        }

        let mut sha256 = Sha256::new();
        sha256.update(&data);
        let checksum: [u8; HASH_SIZE] = sha256.finalize().into();
        data.extend_from_slice(&checksum);

        let mut file = File::create(index_path)?;
        file.write_all(&data)?;

        Ok(())
    }

    pub fn add_file(&mut self, path: &Path) -> io::Result<()> {
        let metadata = path.metadata()?;
        if !metadata.is_file() {
            return Err(Error::new(ErrorKind::Other, "Not a file"));
        }

        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let obj = Object::read_raw_object("blob", &data)?;
        obj.hash_object(true)?;

        let header = format!("blob {}\x00", data.len());
        let mut complete = Vec::new();
        complete.extend_from_slice(header.as_bytes());
        complete.extend_from_slice(&data);

        let mut sha256 = Sha256::new();
        sha256.update(&complete);
        let hash: [u8; HASH_SIZE] = sha256.finalize().into();

        let path_str = path.to_string_lossy().to_string();

        self.entries.retain(|e| e.path != path_str);
        self.entries.push(IndexEntry {
            ctime_secs: metadata.ctime() as u32,
            ctime_nanos: metadata.ctime_nsec() as u32,
            mtime_secs: metadata.mtime() as u32,
            mtime_nanos: metadata.mtime_nsec() as u32,
            dev: metadata.dev() as u32,
            ino: metadata.ino() as u32,
            mode: 0o100644,
            uid: metadata.uid(),
            gid: metadata.gid(),
            file_size: metadata.len() as u32,
            hash,
            flags: path_str.len().min(0xFFF) as u16,
            path: path_str,
        });

        self.entries.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(())
    }
}
