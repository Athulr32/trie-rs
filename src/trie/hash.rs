use std::io::Write;

use crate::{rlp::rlp_encoder::RlpEncoder, trie::encoding::hex_to_compact};

use super::node::{self, FullNode, HashNode, Node, ShortNode};
use sha3::{self, Digest, Sha3_256};
pub struct Hasher {
    rlp_enc: RlpEncoder,
    temp: Vec<u8>,
}

impl Hasher {
    pub fn new() -> Self {
        Self {
            rlp_enc: RlpEncoder {
                ..Default::default()
            },
            temp: Vec::new(),
        }
    }

    /// Collapses a node into a hash node and prepares its replacement.
    ///
    /// This function performs two main operations:
    /// 1. It collapses the input node into a hash representation.
    /// 2. It creates a copy of the original node, but initialized with the computed hash.
    ///
    /// The purpose is to optimize the trie structure by replacing large nodes with their
    /// hash representations while maintaining the ability to reconstruct the original node.
    ///
    /// # Arguments
    ///
    /// * `node` - A reference to the node to be collapsed. The exact type depends on your trie implementation.
    ///
    pub fn hash(&mut self, node: &Node, force: bool) -> (Node, Node) {
        match node {
            Node::FullNode(n) => {
                let (collapsed, mut cached) = self.hash_full_node_children(&n);
                let hashed = self.full_node_to_hash(&collapsed, force);
                if let Node::HashNode(hn) = &hashed {
                    cached.flags.hash = Some(hn.clone());
                } else {
                    cached.flags.hash = None;
                }
                return (hashed, Node::FullNode(cached));
            }
            Node::ShortNode(n) => {
                let (collapsed, mut cached) = self.hash_short_node_children(&n);
                let hashed = self.short_node_to_hash(&collapsed, force);
                if let Node::HashNode(hn) = &hashed {
                    cached.flags.hash = Some(hn.clone());
                } else {
                    cached.flags.hash = None;
                }
                return (hashed, Node::ShortNode(cached));
            }
            _ => {
                // Value and hash nodes don't have children, so they're left as were
                return (node.clone(), node.clone());
            }
        }
    }

    pub fn hash_full_node_children(&mut self, node: &FullNode) -> (FullNode, FullNode) {
        let mut collapsed = node.clone();
        let mut cached = node.clone();

        for i in 0..16 {
            let child = &node.children[i];

            if let Node::Empty = child {
            } else {
                let (collapsed_new, cached_new) = self.hash(child, false);
                collapsed.children[i] = collapsed_new;
                cached.children[i] = cached_new;
            }
        }

        (collapsed, cached)
    }

    /// Collapses the short node.
    ///
    /// # Important
    /// The returned collapsed node holds a live reference to the Key, and must not be modified.
    pub fn hash_short_node_children(&mut self, node: &ShortNode) -> (ShortNode, ShortNode) {
        let mut collapsed = node.clone();
        let mut cached = node.clone();

        collapsed.key = hex_to_compact(&node.key);

        match node.val.as_ref().clone() {
            Node::FullNode(_) | Node::ShortNode(_) => {
                let (collapsed_new, cached_new) = self.hash(&node.val, false);
                collapsed.val = Box::new(collapsed_new);
                cached.val = Box::new(cached_new);
            }

            _ => {}
        }

        (collapsed, cached)
    }

    /// Creates a `HashNode` from a `ShortNode`. The supplied `ShortNode`
    /// should have a hex-type key, which will be converted (without modification)
    /// into compact form for RLP encoding.
    ///
    /// # Returns
    ///
    /// - `Some(HashNode)` if the RLP data is 32 bytes or larger.
    /// - `None` if the RLP data is smaller than 32 bytes.
    pub fn short_node_to_hash(&mut self, node: &ShortNode, force: bool) -> Node {
        node.encode(&mut self.rlp_enc);
        let enc = self.encode_bytes();

        if self.temp.len() > 32 && !force {
            return Node::ShortNode(node.clone());
        }

        Node::HashNode(self.hash_data(&self.temp))
    }

    /// Creates a `HashNode` from a `FullNode`.
    ///
    /// This function converts a `FullNode` into its hash representation. The input
    /// `FullNode` may contain `None` values in its children, which is valid and will
    /// be handled appropriately during the conversion process.
    ///
    /// # Arguments
    ///
    /// * `full_node` - The `FullNode` to be converted to a `HashNode`.
    ///
    /// # Returns
    ///
    /// Returns a `HashNode` representing the hash of the input `FullNode`.
    ///
    /// # Note
    ///
    /// The exact hashing mechanism and handling of `None` values should be
    /// implemented according to the specific requirements of your trie structure.
    pub fn full_node_to_hash(&mut self, node: &FullNode, force: bool) -> Node {
        node.encode(&mut self.rlp_enc);
        let enc = self.encode_bytes();

        if self.temp.len() > 32 && !force {
            return Node::FullNode(node.clone());
        }

        Node::HashNode(self.hash_data(&self.temp))
    }

    /// Returns the result of the last encoding operation on `self.rlp_enc`.
    /// This also resets the encoder buffer.
    ///
    /// All node encoding must be done like this:
    ///
    /// ```
    /// node.encode(&mut self.rlp_enc);
    /// let enc = self.encoded_bytes();
    /// ```
    ///
    /// This convention exists because `node.encode` can only be inlined/escape-analyzed when
    /// called on a concrete receiver type.
    pub fn encode_bytes(&mut self) -> &[u8] {
        self.rlp_enc.append_to_bytes(&mut self.temp);
        self.rlp_enc.reset();

        return &self.temp;
    }

    /// Hashes the provided data.
    pub fn hash_data(&self, data: &Vec<u8>) -> HashNode {
        //FIXME:
        let mut hasher = Sha3_256::new();
        hasher.update(data);

        // read hash digest
        let result = hasher.finalize();

        let hash_node = result.to_vec();

        hash_node
    }
}
