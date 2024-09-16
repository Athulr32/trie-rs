use super::types::{Id, HASH_LENGTH};
use crate::trie::types::Hash;

// TrieID constructs an identifier for a standard trie(not a second-layer trie)
// with provided root. It's mostly used in tests and some other tries like CHT trie.
pub fn trie_id(root: Hash) -> Id {
    Id {
        state_root: root,
        owner: [0;HASH_LENGTH],
        root,
    }
}