use crate::{rlp::rlp_encoder::RlpEncoder, trie::encoding::hex_to_compact};

use super::node::{self, FullNode, HashNode, Node, ShortNode};

pub struct Hasher {
    rlp_enc: RlpEncoder,
}

impl Hasher {
    pub fn hash(&self, node: &Node, force: bool) -> (Node, Node) {
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

    pub fn hash_full_node_children(&self, node: &FullNode) -> (FullNode, FullNode) {
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

    pub fn hash_short_node_children(&self, node: &ShortNode) -> (ShortNode, ShortNode) {
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

    pub fn short_node_to_hash(&self, node: &ShortNode, force: bool) -> Node {
        unimplemented!()
    }

    pub fn full_node_to_hash(&self, node: &FullNode, force: bool) -> Node {
        unimplemented!()
    }
}
