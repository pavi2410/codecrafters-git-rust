use std::fs;

use crate::utils;

pub enum Object {
    Blob { data: Vec<u8> },
    Tree { entries: Vec<TreeEntry> },
}

pub struct TreeEntry {
    pub mode: String,
    pub filename: String,
    pub sha: String,
}

// blob [content size]\0[Object Content]
// tree [content size]\0[Object Entries]
// [mode] [Object name]\0[SHA-1 in binary format]

impl Object {
    pub fn read_object(sha: &str) -> Vec<u8> {
        let (dir, file) = sha.split_at(2);
        let obj_path = format!(".git/objects/{}/{}", dir, file);
        let compressed = fs::read(obj_path).unwrap();
        utils::decompress_bytes(&compressed)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Object::Blob { data } => {
                let mut blob = format!("blob {}\0", data.len()).as_bytes().to_vec();
                blob.append(&mut data.clone());
                blob
            }
            Object::Tree { entries } => {
                let mut tree = Vec::new();
                for entry in entries {
                    tree.append(&mut format!("{} {}\0", entry.mode, entry.filename).as_bytes().to_vec());
                    tree.append(&mut hex::decode(&entry.sha).unwrap());
                }
                tree
            }
        }
    }

    pub fn sha(&self) -> String {
        utils::sha_from_bytes(&self.to_bytes())
    }

    pub fn write_object(&self) {
        let sha = self.sha();

        let (dir, file) = sha.split_at(2);
        let object_filename = format!(".git/objects/{}/{}", dir, file);
        let object_path = std::path::Path::new(&object_filename);
        fs::create_dir_all(object_path.parent().unwrap()).unwrap();

        let compressed = utils::compress_bytes(&self.to_bytes());

        fs::write(object_path, compressed).unwrap();
    }

    pub fn parse_from_file(sha: &str) -> Object {
        let mut obj_file = Self::read_object(sha).into_iter();

        let header = obj_file
            .by_ref()
            .take_while(|c| *c != 0)
            .collect::<Vec<_>>();

        let (obj_type, payload_size) = header.split_at(4);
        let payload_size = String::from_utf8(payload_size[1..].to_owned()).unwrap().parse::<usize>().unwrap(); 
        let payload = obj_file.by_ref().take(payload_size).collect::<Vec<_>>();

        match obj_type {
            b"blob" => {
                Object::Blob { data: payload }
            }
            b"tree" => {

                let entries = parse_tree_entries(payload);
                Object::Tree { entries }
            }
            _ => panic!("Unknown object type: {:#?}", obj_type),
        }
    }
}

fn parse_tree_entries(obj_content: Vec<u8>) -> Vec<TreeEntry> {
    let mut entries = Vec::new();
    let mut i = 0;

    while i < obj_content.len() {
        let mode = obj_content[i..i + 6].to_vec();
        i += 6;

        let filename = {
            let mut filename = Vec::new();
            while obj_content[i] != 0 {
                filename.push(obj_content[i]);
                i += 1;
            }
            i += 1;
            String::from_utf8(filename).unwrap()
        };

        let sha = obj_content[i..i + 20].to_vec();
        i += 20;

        entries.push(TreeEntry {
            mode: String::from_utf8(mode).unwrap(),
            filename,
            sha: utils::sha_from_bytes(&sha),
        });
    }

    entries
}
