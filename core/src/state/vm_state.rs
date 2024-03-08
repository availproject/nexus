use crate::{
    state::MerkleStore,
    traits::Leaf,
    types::{AppAccountId, ShaHasher, SimpleAccountState, SimpleStateUpdate},
};
use anyhow::{anyhow, Error};
use rocksdb::{Options, DB};
use serde::{de::DeserializeOwned, Serialize};
use sparse_merkle_tree::{traits::Hasher, traits::Value, MerkleProof, SparseMerkleTree, H256};
use std::{
    cmp::PartialEq,
    collections::HashMap,
    sync::{Arc, Mutex},
};

//TODO - Replace MerkleStore with a generic so any backing store could be used.
pub struct VmState {
    tree: SparseMerkleTree<ShaHasher, SimpleAccountState, MerkleStore>,
    merkle_store: MerkleStore,
}

impl VmState {
    pub fn new(root: H256, path: &str) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        let db = DB::open(&db_options, path).expect("unable to open rocks db.");
        let cache: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let cache_arc = Arc::new(Mutex::new(cache));
        let db_arc = Arc::new(Mutex::new(db));

        VmState {
            tree: SparseMerkleTree::new(
                root,
                MerkleStore::with_db(db_arc.clone(), cache_arc.clone()),
            ),
            merkle_store: MerkleStore::with_db(db_arc, cache_arc),
        }
    }

    //Revert to last committed state and clear cache.
    pub fn revert(&mut self) -> Result<(), Error> {
        self.merkle_store.clear_cache()?;

        let tree = match SparseMerkleTree::new_with_store(self.merkle_store.clone()) {
            Ok(i) => i,
            Err(e) => {
                return Err(anyhow!(
                    "Could not calculate root from last committed state. Critical error. {e}"
                ))
            }
        };

        self.tree = tree;
        println!("Reverted to root: {:?}", self.tree.root());

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        match self.merkle_store.commit() {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn update_set(
        &mut self,
        set: Vec<(H256, SimpleAccountState)>,
    ) -> Result<SimpleStateUpdate, Error> {
        let pre_state_root = self.get_root();
        let pre_merkle_proof = self
            .tree
            .merkle_proof(set.iter().map(|v| v.0).collect())
            .unwrap();

        let pre_merkle_set = set
            .iter()
            .map(|v| {
                (
                    AppAccountId::from(v.0),
                    self.tree.get(&v.0).expect("Cannot get from tree."),
                )
            })
            .collect();

        self.tree
            .update_all(set.clone().into_iter().map(|v| (v.0, v.1)).collect())
            .unwrap();

        let post_state_root = self.get_root();

        let post_merkle_set = set
            .iter()
            .map(|v| (AppAccountId::from(v.0), v.1.clone()))
            .collect();
        // let post_merkle_proof = self
        //     .tree
        //     .merkle_proof(set.iter().map(|v| v.0).collect())
        //     .unwrap();

        //println!("Pre: {:?} || Post {:?}", pre_merkle_proof, post_merkle_proof);

        Ok(SimpleStateUpdate {
            pre_state_root,
            post_state_root,
            proof: Some(pre_merkle_proof),
            pre_state: pre_merkle_set,
            post_state: post_merkle_set,
        })
    }

    pub fn get(&self, key: &H256, committed: bool) -> Result<Option<SimpleAccountState>, Error> {
        self.merkle_store
            .get(key.as_slice(), committed)
            .map_err(|e| anyhow!({ e }))
    }

    //Gets from state even if not committed.
    pub fn get_with_proof(&self, key: &H256) -> Result<(SimpleAccountState, MerkleProof), Error> {
        let value = match self.tree.get(key) {
            Ok(i) => i,
            Err(_e) => return Err(anyhow!("Erroneous state.")),
        };

        let proof = match self.tree.merkle_proof(vec![*key]) {
            Ok(i) => i,
            Err(_e) => return Err(anyhow!("Erroneous state.")),
        };

        Ok((value, proof))
    }

    pub fn get_root(&self) -> H256 {
        *self.tree.root()
    }
}
