use crate::rlp::{self, encoder_buffer::EMPTY_STRING, rlp_encoder::RlpEncoder};

#[derive(Clone)]
pub enum Node {
    FullNode(FullNode),
    ShortNode(ShortNode),
    HashNode(HashNode),
    ValueNode(ValueNode),
    Empty,
}

static INDICES: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "[17]",
];

impl Node {
    // EncodeRLP encodes a full node into the consensus RLP format.
    pub fn encode_rlp() {}
}

#[derive(Clone)]
pub struct ShortNode {
    pub key: Vec<u8>,
    pub val: Box<Node>,
    pub flags: NodeFlag,
}

pub type HashNode = Vec<u8>;

pub type ValueNode = Vec<u8>;

#[derive(Clone, Default)]
pub struct FullNode {
    pub children: Vec<Node>,
    pub flags: NodeFlag,
}

// rawNode is a simple binary blob used to differentiate between collapsed trie
// nodes and already encoded RLP binary blobs (while at the same time store them
// in the same cache fields).
type RawNode = Vec<u8>;

#[derive(Clone, Default)]
// nodeFlag contains caching-related metadata about a node.
pub struct NodeFlag {
    pub hash: Option<HashNode>, // cached hash of the node (may be nil)
    pub dirty: bool,            // whether the node has changes that must be written to the database
}

impl FullNode {
    pub fn cache(&self) -> (Option<HashNode>, bool) {
        (self.flags.hash.clone(), self.flags.dirty)
    }

}

impl ShortNode {
    pub fn cache(&self) -> (Option<HashNode>, bool) {
        (self.flags.hash.clone(), self.flags.dirty)
    }
}
