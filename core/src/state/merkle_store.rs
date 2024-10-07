use anyhow::{anyhow, Error};
use jmt::storage::{LeafNode, Node, NodeBatch, NodeKey, TreeReader, TreeWriter};
use jmt::{KeyHash, OwnedValue, Version};
use rocksdb::DB;
use rocksdb::{WriteBatch, WriteOptions};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use sparse_merkle_tree::BranchKey;
use sparse_merkle_tree::BranchNode;
use sparse_merkle_tree::H256;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
//Store to be used inside StateMachine to store Merkle Tree.
#[derive(Clone)]
pub struct MerkleStore {
    db: Arc<Mutex<DB>>,
    cache: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MerkleStore {
    pub fn with_db(db: Arc<Mutex<DB>>, cache: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>) -> Self {
        MerkleStore { db, cache }
    }

    pub fn get<V: DeserializeOwned>(
        &self,
        serialized_key: &[u8],
        committed: bool,
    ) -> Result<Option<V>, Error> {
        if !committed {
            let cache = match self.cache.lock() {
                Ok(i) => i,
                Err(e) => return Err(anyhow!("No lock obtained.")),
            };

            match cache.get(serialized_key) {
                Some(i) => {
                    //Empty vectors mean the value was deleted.
                    if !i.is_empty() {
                        Ok(from_slice::<Option<V>>(&i).unwrap())
                    } else {
                        Ok(None)
                    }
                }
                None => {
                    let db = match self.db.lock() {
                        Ok(i) => i,
                        Err(e) => return Err(anyhow!("No lock obtained.")),
                    };

                    match db.get(serialized_key) {
                        Ok(Some(i)) => {
                            let deserialized_value: V = match from_slice(&i) {
                                Ok(v) => v,
                                Err(e) => return Err(anyhow!(e.to_string())),
                            };

                            Ok(Some(deserialized_value))
                        }
                        Ok(None) => Ok(None),
                        Err(e) => Err(anyhow!(e.to_string())),
                    }
                }
            }
        } else {
            let db = match self.db.lock() {
                Ok(i) => i,
                Err(e) => return Err(anyhow!("No lock obtained.")),
            };

            match db.get(serialized_key) {
                Ok(Some(i)) => {
                    let deserialized_value: V = match from_slice(&i) {
                        Ok(v) => v,
                        Err(e) => return Err(anyhow!(e.to_string())),
                    };

                    Ok(Some(deserialized_value))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(anyhow!(e.to_string())),
            }
        }
    }

    pub fn put<V: Serialize>(&self, serialized_key: &[u8], value: &V) -> Result<(), Error> {
        let mut cache = match self.cache.lock() {
            Ok(i) => i,
            Err(e) => return Err(anyhow!("No lock obtained.")),
        };

        cache.insert(serialized_key.to_vec(), to_vec(value).unwrap());

        Ok(())
    }

    pub fn delete(&self, serialized_key: &[u8]) -> Result<(), Error> {
        let mut cache = match self.cache.lock() {
            Ok(i) => i,
            Err(e) => return Err(anyhow!("No lock obtained.")),
        };

        cache.remove(&serialized_key.to_vec());

        cache.insert(serialized_key.to_vec(), vec![]);

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        let db = match self.db.lock() {
            Ok(i) => i,
            Err(e) => return Err(anyhow!("No lock obtained.")),
        };
        let mut cache = match self.cache.lock() {
            Ok(i) => i,
            Err(e) => return Err(anyhow!("No lock obtained.")),
        };

        for (key, value) in cache.iter() {
            if !value.is_empty() {
                match db.put(key, value) {
                    Err(e) => return Err(anyhow!(e.to_string())),
                    _ => (),
                }
            } else {
                //Getting from underlying db below so as to not deserialise the
                //value as it is not required.
                match db.get(key) {
                    Err(e) => return Err(anyhow!(e.to_string())),
                    Ok(Some(_)) => match db.delete(key) {
                        Err(e) => return Err(anyhow!(e.to_string())),
                        _ => (),
                    },
                    Ok(None) => (),
                };
            }
        }

        cache.clear();
        Ok(())
    }

    pub fn clear_cache(&mut self) -> Result<(), Error> {
        let mut cache = match self.cache.lock() {
            Ok(i) => i,
            Err(e) => return Err(anyhow!("No lock obtained.")),
        };

        Ok(cache.clear())
    }
}

impl TreeReader for MerkleStore {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>, anyhow::Error> {
        match self.get(&to_vec(node_key)?, true) {
            Ok(i) => Ok(i),
            Err(e) => Err(anyhow!(e)),
        }
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, LeafNode)>, anyhow::Error> {
        todo!("StateDB does not support [`TreeReader::get_rightmost_leaf`] yet")
    }

    fn get_value_option(
        &self,
        max_version: Version,
        key_hash: KeyHash,
    ) -> Result<Option<OwnedValue>, anyhow::Error> {
        let values: Vec<(Version, OwnedValue)> = match self.get(&key_hash.0, true) {
            Ok(Some(i)) => i,
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => return Err(anyhow!(e)),
        };
        let mut found_value: Option<(Version, OwnedValue)> = None;

        values.into_iter().for_each(|(version, value)| {
            if version <= max_version {
                match found_value {
                    Some((found_version, _)) => {
                        if found_version < version {
                            found_value = Some((version, value))
                        }
                    }
                    None => found_value = Some((version, value)),
                }
            }
        });

        Ok(match found_value {
            Some(i) => Some(i.1),
            None => None,
        })
    }
}

//TODO: Optimise the storage of all version values as a vector to avoid loops.
impl TreeWriter for MerkleStore {
    fn write_node_batch(&self, node_batch: &NodeBatch) -> Result<(), anyhow::Error> {
        let mut batch = WriteBatch::default();

        // Add nodes to the batch
        for (node_key, node) in node_batch.nodes() {
            let serialized_key = to_vec(node_key).map_err(|e| anyhow!(e))?;
            let serialized_value = to_vec(node).map_err(|e| anyhow!(e))?;
            batch.put(serialized_key, serialized_value);
        }

        let mut updates: BTreeMap<KeyHash, Vec<(Version, Option<OwnedValue>)>> = BTreeMap::new();
        for ((version, key_hash), value) in node_batch.values() {
            updates
                .entry(key_hash.clone())
                .or_default()
                .push((*version, value.clone()));
        }

        // Process each key_hash
        for (key_hash, changes) in updates {
            // Retrieve existing values from the database
            let mut existing_values: Vec<(Version, OwnedValue)> = match self.get(&key_hash.0, true)
            {
                Ok(Some(values)) => values,
                Ok(None) => Vec::new(),
                Err(e) => return Err(anyhow!(e)),
            };

            // Apply changes
            for (version, value) in changes {
                if let Some(val) = value {
                    // Check if the version already exists
                    if let Some(existing) = existing_values.iter_mut().find(|(v, _)| v == &version)
                    {
                        // Replace the existing value with the new value
                        existing.1 = val;
                    } else {
                        // Add the new version and value
                        existing_values.push((version, val));
                    }
                } else {
                    // If value is None, remove the entry with the specified version
                    existing_values.retain(|(v, _)| v != &version);
                }
            }

            // Sort values in descending order by version
            existing_values.sort_by(|a, b| b.0.cmp(&a.0));

            // Serialize and insert the updated array
            let serialized_value = to_vec(&existing_values).map_err(|e| anyhow!(e))?;
            batch.put(key_hash.0, serialized_value);
        }

        let db = match self.db.lock() {
            Ok(i) => i,
            Err(e) => return Err(anyhow!("No lock obtained.")),
        };
        // Write the batch atomically
        db.write_opt(batch, &WriteOptions::default())
            .map_err(|e| anyhow!(e))?;

        Ok(())
    }
}
