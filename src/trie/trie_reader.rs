use std::collections::HashMap;

use super::types::{Hash, MissingNodeError, Reader};

pub struct TrieReader {
    pub owner: Hash,
    pub reader: Option<Box<dyn Reader>>,
    pub banned: HashMap<String, ()>,
}

impl TrieReader {
    // pub fn node(&self,path :Vec<u8>, hash : Hash)-> Result<Vec<u8>> {
    //     // Perform the logics in tests for preventing trie node access.
    //     if self.banned != None {
    //         if _, ok := r.banned[string(path)]; ok {
    //             return nil, &MissingNodeError{Owner: r.owner, NodeHash: hash, Path: path}
    //         }
    //     }
    //     if self.reader == None {
    //         return Err(());
    //     }
    //     blob, err := r.reader.Node(r.owner, path, hash)
    //     if err != nil || len(blob) == 0 {
    //         return nil, &MissingNodeError{Owner: r.owner, NodeHash: hash, Path: path, err: err}
    //     }
    //     return blob, nil
    // }
}
