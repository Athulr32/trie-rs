use std::collections::HashMap;

use super::types::{Database, Hash, MissingNodeError, Reader, EMPTY_ROOT_HASH, HASH_LENGTH};

/// A wrapper of the underlying node reader. Not safe for concurrent usage.
#[derive(Default)]
pub struct TrieReader {
    pub owner: Hash,

    /// Database reader
    pub reader: Option<Box<dyn Reader>>,
}

impl TrieReader {
    pub fn node(&self, path: Option<Vec<u8>>, hash: Hash) -> Result<Vec<u8>, ()> {
        if let Some(reader) = &self.reader {
            let blob = reader.node(self.owner, path, hash);

            if blob.is_err() {
                return Err(());
            }

            let blob = blob.unwrap();
            if blob.len() == 0 {
                return Err(());
            }

            return Ok(blob);
        } else {
            return Err(());
        }
    }
}

pub fn new_trie_reader(
    state_root: &Hash,
    owner: &Hash,
    db: &impl Database,
) -> Result<TrieReader, MissingNodeError> {
    if state_root == &[0; HASH_LENGTH] || state_root == &EMPTY_ROOT_HASH {
        if state_root == &[0; HASH_LENGTH] {
            eprint!("Zero state root hash!");
        }
        return Ok(TrieReader {
            owner: *owner,
            reader: None,
        });
    }

    match db.reader(&state_root) {
        Ok(reader) => Ok(TrieReader {
            owner: *owner,
            reader: Some(reader),
        }),
        Err(err) => Err(MissingNodeError {
            owner: *owner,
            node_hash: *state_root,
            err: Box::new(err),
        }),
    }
}

pub fn new_empty_reader() -> TrieReader {
    TrieReader {
        ..Default::default()
    }
}
