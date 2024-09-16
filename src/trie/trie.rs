use std::collections::HashMap;

use super::{
    node::{HashNode, Node}, trie_id::trie_id, trie_reader::TrieReader, types::{Database, Hash, Id, MissingNodeError, Tracer, ADDRESS_LENGTH, EMPTY_ROOT_HASH, HASH_LENGTH}
};

struct Trie {
    pub root: Option<Node>,
    pub owner: Hash,

    // Flag whether the commit operation is already performed. If so the
    // trie is not usable(latest states is invisible).
    pub committed: Option<bool>,

    // Keep track of the number leaves which have been inserted since the last
    // hashing operation. This number will not directly map to the number of
    // actually unhashed nodes.
    pub unhashed: Option<i32>,

    // reader is the handler trie can retrieve nodes from.
    pub reader: TrieReader,

    // tracer is the tool to track the trie changes.
    pub tracer: Tracer,
}



impl std::error::Error for MissingNodeError {}

impl Trie {
    pub fn get(
        &mut self,
        orig_node: Node,
        key: Vec<u8>,
        pos: usize,
    ) -> (Vec<u8>, Node, bool, Option<String>) {
        match orig_node {
            Node::FullNode(node) => {
                let req_pos = key[pos];
                let (value, newnode, did_resolve, err) =
                    self.get(node.children[req_pos as usize].clone(), key, pos + 1);

                return (value, newnode, false, None);
            }
            Node::ValueNode(val) => {
                return (val.clone(), Node::ValueNode(val), false, None);
            }
            Node::HashNode(e) => {
                let new_node = self.resolve_and_track(e, Some(key.clone())).unwrap();

                let (value, newnode, did_resolve, err) = self.get(new_node, key, pos);

                return (value, newnode, did_resolve, err);
            }
            Node::ShortNode(e) => {
                let (value, newnode, did_resolve, err) = self.get(e.val.as_ref().clone(), key, pos);

                return (value, newnode, did_resolve, err);
            }
            _ => panic!("sds"),
        }
    }

    fn resolve_and_track(
        &mut self,
        hashNode: HashNode,
        prefix: Option<Vec<u8>>,
    ) -> Result<Node, std::io::Error> {
        // Implementation of resolve_and_track goes here
        // This is a placeholder and should be replaced with actual logic
        unimplemented!()
    }
}

fn new_trie_reader(
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
            banned: HashMap::new(),
        });
    }

    match db.reader(&state_root) {
        Ok(reader) => Ok(TrieReader {
            owner: *owner,
            reader: Some(reader),
            banned: HashMap::new(),
        }),
        Err(err) => Err(MissingNodeError {
            owner: *owner,
            node_hash: *state_root,
            err: Box::new(err),
        }),
    }
}

/// Creates a new trie instance with the provided trie ID and read-only database.
///
/// # Arguments
///
/// * `id` - The trie ID containing state root, owner, and root hash.
/// * `db` - The database implementing the Database trait.
///
/// # Returns
///
/// Returns a Result containing either the new Trie instance or an error.
pub fn new(id: Id, db: &impl Database) -> Result<Trie, std::io::Error> {
    let reader = new_trie_reader(&id.state_root, &id.owner, db).unwrap();

    let mut trie = Trie {
        owner: id.owner,
        reader,
        tracer: Tracer::default(),
        root: None,
        committed: None,
        unhashed: None,
    };

    if id.root != [0; HASH_LENGTH] && id.root != EMPTY_ROOT_HASH {
        let root_node = trie.resolve_and_track(id.root.to_vec(), None)?;
        trie.root = Some(root_node);
    }

    Ok(trie)
}

// NewEmpty is a shortcut to create empty tree. It's mostly used in tests.
pub fn new_empty(db: &impl Database) -> Trie {
    new(trie_id(EMPTY_ROOT_HASH), db).unwrap()
}
