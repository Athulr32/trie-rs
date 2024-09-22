use std::collections::HashMap;

use super::{
    encoding::prefix_len,
    hash::Hasher,
    node::{must_decode_node, FullNode, HashNode, Node, NodeFlag, ShortNode},
    trie_id::trie_id,
    trie_reader::{new_trie_reader, TrieReader},
    types::{
        Database, Hash, Id, MissingNodeError, Tracer, ADDRESS_LENGTH, EMPTY_ROOT_HASH, HASH_LENGTH,
    },
};

pub struct Trie {
    pub root: Option<Node>,
    pub owner: Hash,

    /// Flag whether the commit operation is already performed. If so the
    /// trie is not usable(latest states is invisible).
    pub committed: Option<bool>,

    /// Keeps track of the number of leaves which have been inserted since the last
    /// hashing operation. This number will not directly map to the number of
    /// actually unhashed nodes.
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

    pub fn insert(
        &mut self,
        node: Node,
        prefix: Vec<u8>,
        key: Vec<u8>,
        value: Node,
    ) -> Result<(bool, Node), ()> {
        if key.len() == 0 {
            if let Node::ValueNode(v) = node {
                if let Node::ValueNode(n) = &value {
                    let comp = v == *n;

                    return Ok((!comp, value));
                }
            }

            return Ok((true, value));
        }

        match node {
            Node::FullNode(mut n) => {
                let index = key[0] as usize;
                let ins = self.insert(n.children[index].clone(), prefix, key, value);

                if let Ok((dirty, nn)) = ins {
                    if !dirty {
                        return Err(());
                    }
                    n.flags = self.new_flag();
                    n.children[index] = nn;

                    return Ok((true, Node::FullNode(n)));
                } else {
                    return Err(());
                }
            }
            Node::HashNode(n) => {
                // We've hit a part of the trie that isn't loaded yet. Load
                // the node and insert into it. This leaves all child nodes on
                // the path to the value in the trie.
                let rt = self.resolve_and_track(n, Some(prefix.clone()));
                if let Ok(rt) = rt {
                    let ins = self.insert(rt, prefix, key, value);
                    if let Ok((dirty, nn)) = ins {
                        return Ok((true, nn));
                    } else {
                        return Err(());
                    }
                } else {
                    return Err(());
                }
            }
            Node::ShortNode(n) => {
                let matchlen = prefix_len(&key, &n.key);

                // If the whole key matches, keep this short node as is
                // and only update the value.
                if matchlen == n.key.len() {
                    let ins = self.insert(n.val.as_ref().clone(), prefix, key, value);

                    if let Ok(ins) = ins {
                        if !ins.0 {
                            return Err(());
                        } else {
                            return Ok((true, Node::ShortNode(n)));
                        }
                    } else {
                        return Err(());
                    }
                }

                let mut branch = FullNode {
                    flags: self.new_flag(),
                    ..Default::default()
                };

                let ins1 = self.insert(
                    Node::Empty,
                    [prefix.clone(), n.key[..matchlen + 1].to_vec()].concat(),
                    n.key.clone(),
                    n.val.as_ref().clone(),
                );

                if let Ok((_, nn)) = ins1 {
                    branch.children[n.key[matchlen] as usize] = nn;
                } else {
                    return Err(());
                }

                let ins2 = self.insert(
                    Node::Empty,
                    [prefix, key[..matchlen + 1].to_vec()].concat(),
                    n.key,
                    n.val.as_ref().clone(),
                );

                if let Ok((_, nn)) = ins2 {
                    branch.children[key[matchlen] as usize] = nn;
                } else {
                    return Err(());
                }

                // Replace this shortNode with the branch if it occurs at index 0.
                if matchlen == 0 {
                    return Ok((true, Node::FullNode(branch)));
                }

                // New branch node is created as a child of the original short node.
                // Track the newly inserted node in the tracer. The node identifier
                // passed is the path from the root node.
                //t.tracer.onInsert(append(prefix, key[:matchlen]...))

                // Replace it with a short node leading up to the branch.
                return Ok((
                    true,
                    Node::ShortNode(ShortNode {
                        flags: self.new_flag(),
                        key: key[..matchlen].to_vec(),
                        val: Box::new(Node::FullNode(branch)),
                    }),
                ));
            }
            Node::Empty => {
                return Ok((
                    true,
                    Node::ShortNode(ShortNode {
                        flags: self.new_flag(),
                        key,
                        val: Box::new(value),
                    }),
                ))
            }
            Node::ValueNode(_) => {
                panic!("PANIICCC I DONT KNOW WHY");
            }
        }
    }

    /// delete returns the new root of the trie with key deleted.
    /// It reduces the trie to minimal form by simplifying
    /// nodes on the way up after deleting recursively.
    pub fn delete(
        &mut self,
        node: Node,
        prefix: Vec<u8>,
        key: Vec<u8>,
    ) -> Result<(bool, Node), ()> {
        match node {
            Node::FullNode(n) => {
                let del = self.delete(
                    n.children[key[0] as usize].clone(),
                    [prefix, key[0].to_ne_bytes().to_vec()].concat(),
                    key[1..].to_vec(),
                );
            }
            Node::HashNode(n) => {
                // We've hit a part of the trie that isn't loaded yet. Load
                // the node and insert into it. This leaves all child nodes on
                // the path to the value in the trie.
                let rn = self.resolve_and_track(n, Some(prefix.clone()));

                if let Ok(rn) = rn {
                    let ins = self.delete(rn.clone(), prefix, key);
                    if let Ok((dirty, nn)) = ins {
                        if !dirty {
                            return Err(());
                        }

                        return Ok((false, rn));
                    }
                } else {
                    return Err(());
                }
            }
            Node::ShortNode(n) => {
                let matchlen = prefix_len(&key, &n.key);
                if matchlen < n.key.len() {
                    return Err(());
                }

                if matchlen == key.len() {
                    return Ok((true, (Node::Empty)));
                }

                // The key is longer than n.Key. Remove the remaining suffix
                // from the subtrie. Child can never be nil here since the
                // subtrie must contain at least two other values with keys
                // longer than n.Key.

                let del = self.delete(n.val.as_ref().clone(), prefix, key);

                if let Ok((dirty, child)) = del {
                    if !dirty {
                        return Err(());
                    }

                    match child {
                        Node::ShortNode(child_n) => {
                            return Ok((
                                true,
                                Node::ShortNode(ShortNode {
                                    flags: self.new_flag(),
                                    key: [n.key, child_n.key].concat(),
                                    val: child_n.val,
                                }),
                            ));
                        }
                        _ => {
                            return Ok((
                                true,
                                Node::ShortNode(ShortNode {
                                    flags: self.new_flag(),
                                    val: Box::new(child),
                                    key: n.key,
                                }),
                            ));
                        }
                    }
                } else {
                    return Err(());
                }
            }
            Node::ValueNode(n) => {
                return Ok((true, Node::Empty));
            }
            Node::Empty => return Err(()),
        }

        Err(())
    }

    pub fn resolve(&mut self, node: Node, prefix: Vec<u8>) -> Result<Node, ()> {
        if let Node::HashNode(v) = &node {
            return self.resolve_and_track(v.to_vec(), Some(prefix));
        }

        Ok(node)
    }

    /// Loads a node from the underlying store with the given node hash and path prefix,
    /// and tracks the loaded node blob in the tracer as the node's original value.
    ///
    /// This function prefers to load the RLP-encoded blob from the database because
    /// it's easier to decode a node than to encode a node to a blob.
    fn resolve_and_track(
        &mut self,
        hash_node: HashNode,
        prefix: Option<Vec<u8>>,
    ) -> Result<Node, ()> {
        let mut hash: Hash = [0; HASH_LENGTH];
        hash.copy_from_slice(&hash_node[..HASH_LENGTH]);

        let blob = self.reader.node(prefix, hash)?;

        Ok(must_decode_node(hash_node, blob))
    }

    pub fn new_flag(&self) -> NodeFlag {
        return NodeFlag {
            dirty: true,
            hash: None,
        };
    }

    /// Calculates the root hash of the given trie.
    fn hash_root(&self) -> (Node, Option<Node>) {
        if self.root.is_none() {
            return (Node::HashNode(Vec::new()), None);
        }

        let mut hasher = Hasher::new();
        let (hashed, cached) = hasher.hash(&self.root.clone().unwrap(), true);

        (hashed, Some(cached))
    }

    /// Returns the root hash of the trie.
    ///
    /// This method does not write to the database and can be used even if the trie
    /// doesn't have an associated database.
    pub fn hash(&mut self) -> Hash {
        let (hashed, cached) = self.hash_root();
        self.root = cached;
        let mut hash: Hash = [0; HASH_LENGTH];
        if let Node::HashNode(v) = hashed {
            hash.copy_from_slice(&v[..HASH_LENGTH]);
        }

        hash
    }

    /// Reset resets the states
    pub fn reset(&mut self) {
        self.root = None;
        self.owner = [0; 32];
        self.unhashed = Some(0);
        self.committed = Some(false);
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
pub fn new(id: Id, db: &impl Database) -> Result<Trie, ()> {
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
