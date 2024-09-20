use crate::utils::hasher::Sha256;
use crate::{
    state::{types::AccountState, MerkleStore},
    traits::Leaf,
    types::{AppAccountId, ShaHasher, StateUpdate},
};
use anyhow::{anyhow, Error};
use jmt::{
    proof::SparseMerkleProof,
    storage::{NodeBatch, TreeUpdateBatch, TreeWriter},
    JellyfishMerkleTree, KeyHash, SimpleHasher, Version,
};
use rocksdb::{Options, DB};
use sparse_merkle_tree::H256;
use std::{
    cmp::PartialEq,
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct VmState {
    merkle_store: MerkleStore,
}

impl VmState {
    pub fn new(path: &str) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);

        let db = DB::open(&db_options, path).expect("unable to open rocks db.");
        let cache: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        let db_arc = Arc::new(Mutex::new(db));
        let cache_arc = Arc::new(Mutex::new(cache));
        let merkle_store = MerkleStore::with_db(db_arc.clone(), cache_arc.clone());

        Self { merkle_store }
    }

    pub fn get_version(&self) -> Result<Option<u64>, anyhow::Error> {
        self.merkle_store.get(b"version", true)
    }

    pub fn update_version(&self, version: u64) -> Result<(), anyhow::Error> {
        self.merkle_store.put(b"version", &version)
    }

    pub fn get_root(&self, version: u64) -> Result<H256, anyhow::Error> {
        let tree: JellyfishMerkleTree<MerkleStore, Sha256> =
            JellyfishMerkleTree::new(&self.merkle_store);
        let root = match tree.get_root_hash_option(version)? {
            Some(i) => H256::from(i.0),
            None => H256::zero(),
        };
        Ok(root)
    }

    pub fn update_set(
        &mut self,
        set: HashMap<H256, Option<AccountState>>,
        version: Version,
    ) -> Result<(TreeUpdateBatch, StateUpdate), Error> {
        let mut pre_state: HashMap<[u8; 32], (Option<AccountState>, SparseMerkleProof<Sha256>)> =
            HashMap::new();
        let pre_state_root = self.get_root(version)?;

        set.iter()
            .try_for_each::<_, Result<(), anyhow::Error>>(|(key, account)| {
                //Note: Do not have to get version minus one, as any version lesser than equal to is retrieved.
                let result = self.get_with_proof(key, version)?;

                pre_state.insert(key.as_fixed_slice().clone(), result);
                Ok(())
            })?;
        // Convert AccountState to Vec<u8> before inserting into the set
        let serialized_set: HashMap<KeyHash, Option<Vec<u8>>> = set
            .into_iter()
            .map(|(key, value)| {
                let serialized_value = value.map(|account_state| account_state.encode());
                (KeyHash(key.as_fixed_slice().clone()), serialized_value)
            })
            .collect();
        let tree: JellyfishMerkleTree<MerkleStore, Sha256> =
            JellyfishMerkleTree::new(&self.merkle_store);

        // Perform the update with the serialized set
        match tree.put_value_set(serialized_set, version) {
            Ok(i) => Ok((
                i.1,
                StateUpdate {
                    pre_state,
                    post_state_root: H256::from(i.0 .0),
                    pre_state_root,
                },
            )),
            Err(e) => Err(e),
        }
    }

    pub fn commit(&mut self, node_batch: &NodeBatch) -> Result<(), Error> {
        self.merkle_store.write_node_batch(&node_batch)
    }

    pub fn get(&self, key: &H256, version: Version) -> Result<Option<AccountState>, Error> {
        let tree: JellyfishMerkleTree<MerkleStore, Sha256> =
            JellyfishMerkleTree::new(&self.merkle_store);

        match tree.get(KeyHash(key.as_fixed_slice().clone()), version) {
            Ok(Some(value)) => match AccountState::decode(&value) {
                Ok(account_state) => Ok(Some(account_state)),
                Err(e) => Err(e.into()),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    //Gets from state even if not committed.
    pub fn get_with_proof(
        &self,
        key: &H256,
        version: Version,
    ) -> Result<(Option<AccountState>, SparseMerkleProof<Sha256>), Error> {
        let tree: JellyfishMerkleTree<MerkleStore, Sha256> =
            JellyfishMerkleTree::new(&self.merkle_store);
        let root = self.get_root(0)?;

        //TODO: Add genesis state so there is no empty root.
        if root == H256::zero() {
            return Ok((None, SparseMerkleProof::new(None, vec![])));
        }
        let proof = tree.get_with_proof(KeyHash(key.as_fixed_slice().clone()), version)?;

        let result = proof.1.verify(
            jmt::RootHash(root.as_fixed_slice().clone()),
            KeyHash(key.as_fixed_slice().clone()),
            proof.0.clone(),
        );

        let value = match proof.0 {
            Some(i) => match AccountState::decode(&i) {
                Ok(account_state) => Some(account_state),
                Err(e) => return Err(e.into()),
            },
            None => None,
        };

        Ok((value, proof.1))
    }
}
