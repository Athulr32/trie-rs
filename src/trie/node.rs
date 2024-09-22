use crate::rlp::{
    self,
    decode::{count_values, split_list, split_string},
    encoder_buffer::EMPTY_STRING,
    rlp_encoder::RlpEncoder,
};

use super::encoding::{compact_to_hex, has_term, hex_to_compact};

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

pub fn must_decode_node(hash: Vec<u8>, buff: Vec<u8>) -> Node {
    let n = decode_node(hash, buff).expect("Not Expected to Fail");
    n
}

pub fn decode_node(hash: Vec<u8>, buff: Vec<u8>) -> Result<Node, ()> {
    if buff.len() == 0 {}

    let (elems, _) = split_list(&buff[..]).unwrap();

    let count = count_values(&elems).unwrap();

    match count {
        2 => {
            let node = decode_short(hash, elems)?;

            return Ok(Node::ShortNode(node));
        }
        17 => {
            let node = decode_full(hash, elems)?;
            return Ok(Node::FullNode(node));
        }
        _ => Err(()),
    }
}

pub fn decode_short(hash: Vec<u8>, elems: &[u8]) -> Result<ShortNode, ()> {
    let (content, rest) = split_string(elems)?;

    let flag = NodeFlag {
        hash: Some(hash),
        ..Default::default()
    };
    let key = compact_to_hex(content);

    if has_term(&key) {
        //value node
        let (val, _) = split_string(rest)?;

        return Ok(ShortNode {
            key,
            val: Box::new(Node::ValueNode(val.to_vec())),
            flags: flag,
        });
    }

    let (node, _) = decode_ref(rest)?;

    return Ok(ShortNode {
        key,
        val: Box::new(node),
        flags: flag,
    });
}

pub fn decode_full(hash: Vec<u8>, mut elems: &[u8]) -> Result<FullNode, ()> {
    let mut node = FullNode {
        flags: NodeFlag {
            hash: Some(hash),
            ..Default::default()
        },
        ..Default::default()
    };

    for i in 0..16 {
        let (child_node, rest) = decode_ref(elems)?;

        node.children[i] = child_node;
        elems = rest;
    }

    let (val, _) = split_string(elems)?;

    if val.len() > 0 {
        node.children[16] = Node::ValueNode(val.to_vec());
    }

    return Ok(node);
}

fn decode_ref(buff: &[u8]) -> Result<(Node, &[u8]), ()> {
    unimplemented!()
}
