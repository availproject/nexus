use jmt::mock::{self, put_value};
use jmt::storage::{LeafNode, TreeWriter};
use jmt::{mock::MockTreeStore, JellyfishMerkleTree};
use jmt::{KeyHash, OwnedValue, Sha256Jmt, TransparentHasher, ValueHash};
use nexus_core::state::sparse_merkle_tree::merkle_proof::CompiledMerkleProof;
use nexus_core::state::sparse_merkle_tree::traits::Value;
use nexus_core::state::{
    types::{AccountState, StatementDigest},
    vm_state::VmState,
};
use nexus_core::types::{AppAccountId, AppId, ShaHasher, H256};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // let reader = MockTreeStore::new(true);
    // let tree: Sha256Jmt<MockTreeStore> = JellyfishMerkleTree::new(&reader);

    // let key = KeyHash([1u8; 32]);
    // let key_2 = KeyHash([8u8; 32]);
    // let key_3 = KeyHash([3u8; 32]);
    // let key_4 = KeyHash([15u8; 32]);
    // let mut value_history: HashMap<KeyHash, Option<Vec<u8>>> = HashMap::new();

    // value_history.insert(key.clone(), Some(b"123".to_vec()));
    // value_history.insert(key_2.clone(), Some(b"123".to_vec()));
    // value_history.insert(key_3.clone(), Some(b"123".to_vec()));
    // value_history.insert(key_4.clone(), Some(b"123".to_vec()));

    // let result = tree.put_value_set(value_history, 0).unwrap();

    // let update = reader.write_node_batch(&result.1.node_batch).unwrap();
    // let proof = tree.get_with_proof(key, 0).unwrap();
    // let verify = proof.1.verify(result.0, key, Some(b"123".to_vec()));
    // let leaf = LeafNode::new(
    //     KeyHash([3u8; 32]),
    //     ValueHash::with::<sha2::Sha256>(b"123".to_vec()),
    // );

    println!(
        "{:?}",
        hex::encode([
            104, 142, 148, 165, 30, 229, 8, 169, 94, 118, 18, 148, 175, 183, 166, 0, 75, 67, 44,
            21, 217, 137, 12, 128, 221, 242, 59, 222, 140, 170, 76, 38,
        ])
    );
    //let internal_node = Internal
    // let hash = hex::encode(leaf.hash::<sha2::Sha256>());

    // let siblings: Vec<String> = proof
    //     .1
    //     .siblings()
    //     .iter()
    //     .map(|s| hex::encode(s.hash::<sha2::Sha256>()))
    //     .collect();

    // println!(
    //     "{:?} \n {}, \n{:?} \n{:?} \n {:?}",
    //     result.0, hash, siblings, proof.1, key
    // );

    //let siblings = proof.1.siblings().iter().map(|s| s.hash());
    // let proof = CompiledMerkleProof(
    //     [76, 79, 254, 81, 254, 8, 236, 171, 189, 51, 113, 172, 236, 13, 108, 69, 245, 137, 227, 15, 160, 189, 107, 157, 24, 144, 190, 91, 168, 213, 237, 160, 50, 43, 122, 128, 202, 205, 76, 245, 82, 138, 75, 139, 169, 93, 235, 158, 209, 235, 157, 164, 80, 97, 173, 21, 228, 63, 59, 201, 63, 138, 93, 141, 235, 127, 60, 63, 17, 79, 1].to_vec()
    // );
    // println!("{:?}", &proof.compute_root::<ShaHasher>(
    //     vec![
    //         (
    //             H256::from([150, 32, 244, 15, 103, 130, 63, 61, 57, 239, 35, 178, 178, 73, 97, 75, 74, 110, 216, 160, 181, 215, 13, 13, 37, 33, 121, 187, 232, 195, 4, 229]),
    //             H256::from([245, 62, 146, 181, 54, 179, 225, 223, 13, 22, 150, 157, 37, 142, 160, 81, 204, 220, 249, 112, 64, 132, 67, 65, 243, 44, 235, 194, 111, 140, 30, 181])
    //         )
    //     ]
    // ))
    // let old_state_root = H256::zero();
    // let mut vm_state = VmState::new(old_state_root, "./db/chain_state");

    // let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as u32;
    // let app_id = AppAccountId::from(AppId(timestamp));
    // let state = AccountState {
    //     statement: StatementDigest([2601916044, 319464991, 4191464499, 1822444315, 3957549360, 1242837026, 144056965, 2080977897]),
    //     state_root: [156, 142, 175, 73, 63, 139, 78, 220, 226, 186, 22, 71, 52, 62, 173, 204, 9, 137, 207, 70, 30, 113, 44, 10, 98, 83, 255, 44, 161, 132, 43, 183],
    //     start_nexus_hash: [205, 128, 129, 236, 230, 51, 185, 73, 35, 41, 139, 226, 154, 104, 56, 25, 119, 60, 87, 138, 94, 79, 11, 182, 49, 204, 83, 184, 138, 209, 66, 124],
    //     last_proof_height: 0,
    //     height: 183
    // };
    // let timestamp2: u32 = (SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() + 1).try_into().unwrap();
    // let app_id2 = AppAccountId::from(AppId(timestamp2));
    // {
    //     let tree = vm_state.get_tree();

    //     println!("{:?} ", &tree.is_empty(),  );
    // }

    //    {
    //     vm_state.update_set(
    //         vec![(H256::from(app_id.0.clone()), state.clone()), (H256::from(app_id2.0.clone()), state.clone())]
    //         ).unwrap();
    // }

    //     {
    //         let tree = vm_state.get_tree();

    //         println!("\n \n{:?}  \n \n{:?} \n \n{:?}  \n \n{:?}",
    //         &tree.root().as_slice(),
    //         vm_state.get_with_proof(&H256::from(app_id.0.clone())).unwrap().1.compile(vec![H256::from(app_id.0.clone())]).unwrap(),
    //         &app_id.0,
    //         state.to_h256().as_slice(),
    //         );
    //     }

    // ()
}
