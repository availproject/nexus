use nexus_core::{
    db::NodeDB,
    state::vm_state::VmState,
    state_machine::StateMachine,
    types::{
        AvailHeader, DataLookup, Digest, DigestItem, Extension, HeaderStore, KateCommitment,
        NexusHeader, TransactionV2, V3Extension, H256,
    },
    zkvm::ProverMode,
};
use nexus_host::execute_batch;
use rocksdb::Options;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "sp1"))]
use env_logger;

#[cfg(any(feature = "sp1"))]
use log;

/*

    txs: &Vec<TransactionV2>,
    state_machine: &mut StateMachine<E, P>,
    header: &AvailHeader,
    header_store: &mut HeaderStore,

*/

// fn dummy_extension() -> Extension {
//     let app_lookup = DataLookup {
//         size : 0,
//         index: Vec::new(),
//     };
//     let commitment = KateCommitment{
//        rows : 0,
//        cols : 0,
//        commitment: Vec::new(),
//        data_root:H256::zero(),
//     };

//     Extension::V3(V3Extension {
//         app_lookup,
//         commitment,
//     })
// }

fn create_mock_data() -> (
    Vec<TransactionV2>,
    StateMachine<ZKVM, Proof>,
    AvailHeader,
    HeaderStore,
) {
    let _node_db: NodeDB = NodeDB::from_path(&String::from("./db/node_db"));
    let mut db_options = Options::default();
    db_options.create_if_missing(true);

    let state = Arc::new(Mutex::new(VmState::new(&String::from("./db/runtime_db"))));
    let txs: Vec<TransactionV2> = Vec::new();
    let state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());

    let header = AvailHeader {
        parent_hash: H256::from([
            0x05, 0x27, 0x5a, 0xed, 0xae, 0xc8, 0xd5, 0xa6, 0x67, 0x46, 0x4f, 0x46, 0x07, 0x97,
            0xaf, 0xb3, 0x29, 0x36, 0x59, 0x6e, 0xd6, 0xa3, 0xba, 0x1c, 0x86, 0x6f, 0xe9, 0x4f,
            0xd3, 0xfc, 0x50, 0x3c,
        ]),
        number: 10347,
        state_root: H256::from([
            0xf8, 0x6f, 0x0c, 0x2b, 0x31, 0x51, 0x12, 0x8d, 0xca, 0xcd, 0x2d, 0x87, 0x86, 0x77,
            0x87, 0x6b, 0xa6, 0x67, 0xcb, 0x65, 0xd8, 0x84, 0xdb, 0xfc, 0x6a, 0x97, 0xb8, 0xf9,
            0x6b, 0x2f, 0x71, 0xed,
        ]),
        extrinsics_root: H256::from([
            0xf7, 0x16, 0x9a, 0xfe, 0x60, 0xdd, 0x1b, 0xcd, 0x30, 0x9b, 0xf4, 0x5c, 0x37, 0xd4,
            0x75, 0x07, 0xf4, 0xe6, 0xf0, 0xc3, 0x13, 0xce, 0x5b, 0xc6, 0xaf, 0x82, 0xc3, 0x88,
            0xec, 0x6f, 0xe2, 0x0e,
        ]),
        digest: Digest {
            logs: vec![
                DigestItem::PreRuntime(
                    [66, 65, 66, 69],
                    vec![
                        3, 6, 0, 0, 0, 122, 3, 26, 5, 0, 0, 0, 0, 96, 34, 21, 70, 246, 103, 226,
                        78, 207, 90, 117, 4, 179, 81, 213, 146, 85, 114, 19, 39, 197, 48, 119, 143,
                        133, 225, 170, 171, 206, 215, 82, 99, 9, 230, 50, 26, 161, 22, 145, 194,
                        70, 55, 53, 141, 60, 217, 78, 97, 53, 237, 234, 242, 72, 20, 146, 132, 91,
                        126, 221, 69, 250, 112, 91, 10, 30, 194, 194, 9, 150, 122, 244, 105, 114,
                        214, 175, 41, 250, 37, 37, 181, 211, 35, 192, 166, 232, 107, 190, 29, 249,
                        240, 46, 71, 136, 29, 124, 7,
                    ],
                ),
                DigestItem::Seal(
                    [66, 65, 66, 69],
                    vec![
                        58, 161, 53, 108, 139, 73, 153, 22, 220, 31, 123, 19, 92, 31, 154, 93, 17,
                        126, 125, 58, 35, 178, 29, 87, 236, 172, 11, 236, 30, 160, 233, 73, 88, 37,
                        15, 101, 212, 192, 132, 56, 228, 248, 157, 197, 156, 246, 18, 185, 253, 89,
                        253, 175, 194, 64, 127, 247, 128, 74, 45, 161, 88, 126, 205, 139,
                    ],
                ),
            ],
        },
        extension: Extension::V3(V3Extension {
            app_lookup: DataLookup {
                size: 0,
                index: vec![],
            },
            commitment: KateCommitment {
                rows: 1,
                cols: 4,
                commitment: vec![
                    184, 190, 162, 107, 58, 113, 40, 221, 5, 5, 53, 57, 199, 125, 5, 111, 26, 2,
                    97, 216, 87, 163, 118, 228, 9, 132, 182, 100, 103, 34, 162, 81, 7, 254, 182,
                    146, 3, 158, 40, 8, 73, 100, 124, 144, 153, 175, 31, 12, 184, 190, 162, 107,
                    58, 113, 40, 221, 5, 5, 53, 57, 199, 125, 5, 111, 26, 2, 97, 216, 87, 163, 118,
                    228, 9, 132, 182, 100, 103, 34, 162, 81, 7, 254, 182, 146, 3, 158, 40, 8, 73,
                    100, 124, 144, 153, 175, 31, 12,
                ],
                data_root: H256::from([
                    0xad, 0x32, 0x28, 0xb6, 0x76, 0xf7, 0xd3, 0xcd, 0x42, 0x84, 0xa5, 0x44, 0x3f,
                    0x17, 0xf1, 0x96, 0x2b, 0x36, 0xe4, 0x91, 0xb3, 0x0a, 0x40, 0xb2, 0x40, 0x58,
                    0x49, 0xe5, 0x97, 0xba, 0x5f, 0xb5,
                ]),
            },
        }),
    };

    let header_store = HeaderStore {
        inner: vec![
            // NexusHeader {
            //     parent_hash: H256::try_from("484fab2652f9d9ada924943705a38c6170aa0bb337a44d6d9ac0d3f5a81466cc").unwrap(),
            //     prev_state_root: H256::try_from("a53ae1f4c87243a95bb1884d9a7faad6c6274224447f5e758bfe59404b798d09").unwrap(),
            //     state_root: H256::try_from("a53ae1f4c87243a95bb1884d9a7faad6c6274224447f5e758bfe59404b798d09").unwrap(),
            //     avail_header_hash: H256::try_from("616020555dd2bf243029bbe8f49e8104f69fb7a49769d3d945dd75d5f558a8e5").unwrap(),
            //     number: 347,
            // },
            NexusHeader {
                parent_hash: H256::try_from(
                    "f2b8b7095fd3cfb07bb7a32845db8bc84ee835e270e78f9e22c00c680e33a18f",
                )
                .unwrap(),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .unwrap(),
                state_root: H256::try_from(
                    "a53ae1f4c87243a95bb1884d9a7faad6c6274224447f5e758bfe59404b798d09",
                )
                .unwrap(),
                avail_header_hash: H256::try_from(
                    "05275aedaec8d5a667464f460797afb32936596ed6a3ba1c866fe94fd3fc503c",
                )
                .unwrap(),
                number: 346,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "0d943a55b4fba7d05e9e9d8f4b08abdc177492bdb828ec8a1719ec836ffef177",
                )
                .unwrap(),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .unwrap(),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .unwrap(),
                avail_header_hash: H256::try_from(
                    "cb9f15b6375cc1dd20a19e716f36524026111215913dc83c8c55f19a93db64f3",
                )
                .unwrap(),
                number: 345,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "7cebd9b26325bef8044ee8cbbe0941048f00885c24f4f8dbe1458ad187f28cf3",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "82669a2e8d67e8a9cdc17b5ad327545842c692e9e7d0df9f23e975c66eb9c9a4",
                )
                .expect("Invalid hex string"),
                number: 344,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "6e7f942cedbf08815155ed645d4053859666cc3dd215d220c6dcd9fc03826aef",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "c58922dd13db43d0e12e16bde3f781e86b98cea3c9d6edf6b884c2a7c267d39b",
                )
                .expect("Invalid hex string"),
                number: 343,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "023e8162bf6cf17b46c7db74f44bbabb4622c28d609e0fc17da28876b77f1a9b",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "fd7e9b539f2a1696d51f702a74f6ee1cf93e9910aa17fecb9dce8bcfae2a9744",
                )
                .expect("Invalid hex string"),
                number: 342,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "3b1268f154d815c93b5bdd2515610955d708b087ffdc12cf391e65dea2caff79",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "b94406176bb7da38ef356a0d667c566074ca9f5f050d5931374bdb2b9bcee599",
                )
                .expect("Invalid hex string"),
                number: 341,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "f66acde2e24d6e496b167d1e18b2e5770b857166841e004c76b437680df1e1f8",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "8a1615dfff681e6bb97f210cc6e505bd45feb8239270081d053c37937436a4e4",
                )
                .expect("Invalid hex string"),
                number: 340,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "9bc30e78a2acbeb72cb5bc148543886597ca7184b5bfe4c18da13fb1fd8f10ff",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "c153acbc9480befb9c473106fea905bec4d7171e263445501b75c2173a580f5a",
                )
                .expect("Invalid hex string"),
                number: 339,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "6f104953ad8e7d02a41d35854c3db4536d4a6ac10860682b0e5ca51b72ae63bb",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "958beeadcb99105e5896224b2fb530759940299e071cc309db3fa69785f34140",
                )
                .expect("Invalid hex string"),
                number: 338,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "bcb3425b3fe31fe00464c988227fd9eb33cb2d3d2226ad53daf2ce9f68217c28",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "4fca1409ed7646c5f1dfafe4453d0a8736fff16d93774712649c23d50b6dade3",
                )
                .expect("Invalid hex string"),
                number: 337,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "bfcd88237c25333269c5007ac7558c8d8ac3d2b271e922099793af00542c28ac",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "bc9d0c606e38ede6ccec4b836af3bf95b63eefc5c861433309ea1f1e58ab7843",
                )
                .expect("Invalid hex string"),
                number: 336,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "2736fafd0af6bb9335d520d0312bb975f680fed26dbc93191289440282910297",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "62a28b650b51594ac4efb04f12228d9a211e62f86afad322c88c7f69cc56cbf4",
                )
                .expect("Invalid hex string"),
                number: 335,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "657c67230efaaa5ee8b6e97b38fed6eac9bf8c692a1de4e70e5be0c8b7b4fa3e",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "36640470dec26e268041fda5d4eca33a33f247778b5ef9dd601309f07efd2373",
                )
                .expect("Invalid hex string"),
                number: 334,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "ff29acd119dfe111ecd0b367101dfdd0847744c1cba060a70f80efbbbaaac6b4",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "dd2ee7ce5d155f96b82cd490f7705870c69e59b533265ca5145ad3be9a576dab",
                )
                .expect("Invalid hex string"),
                number: 333,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "05a59de14c0ead056e038cd60c710cd510e7aaee92317392a4f64b3a6c818221",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "a40b72bccea9922b7150ab27d712b25060d76867561c809d0f32c53642eea01e",
                )
                .expect("Invalid hex string"),
                number: 332,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "f1a3dd09157208e915af897ec1cb32639bd7e6d5979362007460628200e0812b",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "6f724ecfe82b78c121330fb349113f9de3cf1d8a17c1a23cbad684fdbf6f715a",
                )
                .expect("Invalid hex string"),
                number: 331,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "a70881671f5b65b241ebc6e8a52179a376d6bfa61dd210bd177514e8661c4ee4",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "26878960d149beb56c99d1d1551b348fb138e8aa52c087e970b48df4c12d6d76",
                )
                .expect("Invalid hex string"),
                number: 330,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "e9d89bb459a2c2dbd3c1865558e999f776d0beb2485768398fe51b56dea32a0a",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "8bdfa3db5aa77d59e167521cd664d27eba1cddf6d828cd397394d2851949eb02",
                )
                .expect("Invalid hex string"),
                number: 329,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "aab1f93a4e97e060cd424d1d8d26fe6ef1362222270424dd26770f249abe7609",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "023efa83af944ca119b94a2ac73391d1850f09b359397578d5dd4a8196ec990b",
                )
                .expect("Invalid hex string"),
                number: 328,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "240f10b6ab067509f227557ed63cd1c8ddd5fd98a7d412f2b305f61408e12979",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "9700041913324cadd4b27792c27e4b0d3c95183bf25654365965ba7793fd1944",
                )
                .expect("Invalid hex string"),
                number: 327,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "313458a9d99ab8b60f39864c3360926894d17708d29bd76dafe3e2c056ea87cb",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "b1ec9751b88592e62d2279a1230b3a725a019409bebd0b8569667545662deb00",
                )
                .expect("Invalid hex string"),
                number: 326,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "71d35bf489ea5cd356936de06ba840d068bf482294c42a653d0cdd5752e44358",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "34fb7d340b70907bda4cbee3749b06f4723560168244a0df1d36aca7688c1980",
                )
                .expect("Invalid hex string"),
                number: 325,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "24d4f8ac621f59307f635c4b40e2fd8432c1c2a1d34ac21973564a88cf20cdc2",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "d223742a5d981c08dfdacaf1a966170b09040ef397336206027eb19fc1bb22ee",
                )
                .expect("Invalid hex string"),
                number: 324,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "affa3b552fd0802154a2ffc042aaf5f937e38f6e59ead4feeefde5e2b59e4b33",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "896342e5c187215875c5bb877f1491d54bb82fc0954054b597c881e339950d57",
                )
                .expect("Invalid hex string"),
                number: 323,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "7f79473352a1a0c18bc95fe53b8799a91b1952bab7fb274a323600c451fd90aa",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "468a46d12d897f749c051e2a1986658bc45e1a99f4615ddd2a4012d5767d8675",
                )
                .expect("Invalid hex string"),
                number: 322,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "57f59646610d3fb478aeb3a7c014502017723d60f64e93bb57d93e8a91bf3b89",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "d206f7c643cb8cc4705843e98af2b91e366e5a30dbdde6441bd6e4dee93b6150",
                )
                .expect("Invalid hex string"),
                number: 321,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "ced684bfa3765b96b365653f5d036268196f8c63eb795e8d131d963d4789c6d9",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "26af56d414265a1b769baf86eaa9c0f672d924f8ad485345985572de83945aa2",
                )
                .expect("Invalid hex string"),
                number: 320,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "279ba3106c5c7945d98a9f24ae5e38dbed0ab9a8bcdb13805ca6b5a63cb8a6d5",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "e7520d8f62e571cd60489efc97bd17a9ef2d4d48fa0fe01b79bc4aca8ae04fb5",
                )
                .expect("Invalid hex string"),
                number: 319,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "36d340047ccde4d578ad0114dfad2053bea56cb4cdab55ee54bb5054ea337faf",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "2ff47e5e128c86d9fa15b505604187832bfe6e5fc9e7b957e234e7ee6110b91a",
                )
                .expect("Invalid hex string"),
                number: 318,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "8db74a32fe191112993d74d9ab7859c34abcdd46d80ea15a22a35fb139cedcf1",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "c033d6ad510da2dd7748e0ad6084f647d4f0850ce5614098c79b64c5b292e492",
                )
                .expect("Invalid hex string"),
                number: 317,
            },
            NexusHeader {
                parent_hash: H256::try_from(
                    "cfc08355ae8a6594aa0742350489751d4253e24a59ae9663bef1f56d8c6dbfef",
                )
                .expect("Invalid hex string"),
                prev_state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                state_root: H256::try_from(
                    "258c1d98807e614076861e5c92954ed4d9a341d0b5ecd2c55512b2745489323d",
                )
                .expect("Invalid hex string"),
                avail_header_hash: H256::try_from(
                    "2734a700b7a53e3037aa86234ec551c6309c2666f33d92dcb610123c862cb4bd",
                )
                .expect("Invalid hex string"),
                number: 316,
            },
        ],
        max_size: 32,
    };

    (txs, state_machine, header, header_store)
}

fn main() {
    #[cfg(any(feature = "sp1"))]
    env_logger::Builder::from_env("RUST_LOG")
        .filter_level(log::LevelFilter::Info)
        .init();

    
    let vec = vec![ProverMode::NoAggregation, ProverMode::Compressed];
    
    for i in 0..2 {
        let (txs, mut state_machine, header, mut header_store) = create_mock_data();
        let prover_mode = &vec[i.clone()];

        let start = Instant::now();

        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            // Await the result of `execute_batch`
            let (proof, header) = execute_batch::<Prover, Proof, ZKVM>(
                &txs,
                &mut state_machine,
                &header,
                &mut header_store,
                prover_mode.clone(),
            )
            .await
            .unwrap();

            let duration = start.elapsed();
            println!("Proof generation took: {:?}", duration);

            let current_dir = env::current_dir().unwrap();
            let mut out_sr_path = PathBuf::from(current_dir);
            #[cfg(feature = "risc0")]
            out_sr_path.push("succinct_receipt_risc0.bin");

            #[cfg(feature = "sp1")]
            out_sr_path.push("succinct_receipt_sp1.bin");
            let serialized_data = bincode::serialize(&proof).unwrap();
            let _ = fs::write(out_sr_path.clone(), serialized_data).unwrap();

            let metadata = fs::metadata(&out_sr_path).unwrap();
            let file_size = metadata.len();
            println!("Size of the binary file: {} bytes", file_size);
        })
    }
}
