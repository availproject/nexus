use crate::types::H256;

pub trait Leaf<K> {
    fn get_key(&self) -> K;
}

pub trait NexusTransaction {
    fn hash(&self) -> H256;
}
