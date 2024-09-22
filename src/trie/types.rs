use std::{collections::HashMap, fmt};

pub const HASH_LENGTH: usize = 32;

pub const ADDRESS_LENGTH: u32 = 20;

// Hash represents the 32 byte Keccak256 hash of arbitrary data.
pub type Hash = [u8; HASH_LENGTH];

pub const EMPTY_ROOT_HASH: Hash = [0; HASH_LENGTH];

#[derive(Default, Debug)]
pub struct Tracer {
    inserts: HashMap<String, ()>,
    deletes: HashMap<String, ()>,
    access_list: HashMap<String, Vec<u8>>,
}

pub trait Reader {
    fn node(&self, owner: Hash, path: Option<Vec<u8>>, hash: Hash) -> Result<Vec<u8>, ()>;
}

// ID is the identifier for uniquely identifying a trie.
pub struct Id {
    pub state_root: Hash, // The root of the corresponding state(block.root)
    pub owner: Hash,      // The contract address hash which the trie belongs to
    pub root: Hash,       // The root hash of trie
}

// Database trait
pub trait Database {
    // Reader returns a node reader associated with the specific state.
    // An error will be returned if the specified state is not available.
    fn reader(&self, state_root: &Hash) -> Result<Box<dyn Reader>, std::io::Error>;
}

#[derive(Debug)]
pub struct MissingNodeError {
    pub owner: Hash,
    pub node_hash: Hash,
    pub err: Box<dyn std::error::Error>,
}

impl std::fmt::Display for MissingNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Missing node error: owner {:?}, node hash {:?}",
            self.owner, self.node_hash
        )
    }
}
