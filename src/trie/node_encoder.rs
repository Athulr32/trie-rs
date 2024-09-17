use crate::rlp::{encoder_buffer::EMPTY_STRING, rlp_encoder::RlpEncoder};

use super::node::{FullNode, Node, ShortNode};


impl FullNode {
    pub fn encode(&self, rlp_enc: &mut RlpEncoder) {
        let offset = rlp_enc.list();
        //Encode all the children in the Full Node
        for child in self.children.clone() {
            child.encode(rlp_enc);
        }

        rlp_enc.list_end(offset);
    }
}

impl ShortNode {
    pub fn encode(&self, rlp_enc: &mut RlpEncoder) {
        let offset = rlp_enc.list();
        rlp_enc.write(self.key.clone());

        self.val.as_ref().encode(rlp_enc);

        rlp_enc.list_end(offset);
    }
}

impl Node {
    pub fn encode(&self, rlp_enc: &mut RlpEncoder) {
        match self {
            Node::FullNode(n) => {
                n.encode(rlp_enc);
            }
            Node::ShortNode(short_node) => {
                short_node.val.encode(rlp_enc);
            }
            Node::HashNode(n) => {
                rlp_enc.write_bytes(n.to_vec());
            }
            Node::ValueNode(vn) => {
                rlp_enc.write_bytes(vn.to_vec());
            }
            Node::Empty => {
                rlp_enc.write(EMPTY_STRING.to_vec());
            }
        }
    }
}
