use zksync_core::{STF};
#[cfg(feature = "risc0")]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(feature = "sp1")]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nexus_core::types::{AccountState, AppAccountId, H256 as NexusH256,StatementDigest};
use primitive_types::{H256};
use zksync_core::types::{L1BatchWithMetadata, L1BatchNumber , L1BatchMetadata , L1BatchHeader ,BaseSystemContractsHashes , L1BatchMetaParameters};
use nexus_core::zkvm::ProverMode;
use zksync_basic_types::protocol_version::ProtocolVersionId;
#[cfg(feature = "risc0")] 
use zksync_methods::{ZKSYNC_ADAPTER_ELF, ZKSYNC_ADAPTER_ID};

fn create_mock_data() -> (
    Option<Proof>,
    Option<(AppAccountId, AccountState)>,
    Vec<String>,
    L1BatchWithMetadata,
    Vec<u8>,
    Vec<[u8; 32]>,
    NexusH256,
) {
    let prev_adapter_proof = None; 
    let init_account = Some((
        AppAccountId([1u8; 32]),
        AccountState {
            statement: StatementDigest([3u32; 8]),
            state_root: [1u8; 32],
            start_nexus_hash: [2u8; 32],
            last_proof_height: 0,
            height: 0,
        },
    ));
    let new_rollup_proof = vec!["mock_proof".to_string()];
    let new_rollup_pi = L1BatchWithMetadata {
        header: L1BatchHeader::new(L1BatchNumber(1),0, BaseSystemContractsHashes { bootloader:H256::zero(),default_aa: H256::zero()} , ProtocolVersionId::Version0,),
        metadata: L1BatchMetadata {
            root_hash: H256::zero(),
            rollup_last_leaf_index: 1000,
            initial_writes_compressed: Some(vec![]),
            repeated_writes_compressed: Some(vec![]),
            commitment: H256::zero(),
            l2_l1_merkle_root: H256::zero(),
            block_meta_params: L1BatchMetaParameters{zkporter_is_available: false , bootloader_code_hash: H256::zero() , default_aa_code_hash: H256::zero() , protocol_version: None},
            aux_data_hash: H256::zero(),
            meta_parameters_hash: H256::zero(),
            pass_through_data_hash: Default::default(),
            events_queue_commitment: Some(H256::zero()),
            bootloader_initial_content_commitment: Some(H256::zero()),
            state_diffs_compressed: vec![],
        },
        raw_published_factory_deps: vec![],
    };
    let pubdata_commitments = vec![0u8; 10];
    let versioned_hashes = vec![[0u8; 32]; 5];
    let nexus_hash = NexusH256::zero();

    (
        prev_adapter_proof,
        init_account,
        new_rollup_proof,
        new_rollup_pi,
        pubdata_commitments,
        versioned_hashes,
        nexus_hash,
    )
}

fn create_proof(c: &mut Criterion) {
    let (
        prev_adapter_proof,
        init_account,
        new_rollup_proof,
        new_rollup_pi,
        pubdata_commitments,
        versioned_hashes,
        nexus_hash,
    ) = create_mock_data();
    
    #[cfg(feature = "sp1")]
    let ZKSYNC_ADAPTER_ELF: &[u8] = include_bytes!("../../methods/sp1-guest/elf/riscv32im-succinct-zkvm-elf");
    #[cfg(feature = "sp1")]
    let ZKSYNC_ADAPTER_ID = [0u32; 8]; 

    let img_id = ZKSYNC_ADAPTER_ID;
    let elf = ZKSYNC_ADAPTER_ELF.to_vec(); // Mock ELF data
    let prover_mode = ProverMode::MockProof;

    let stf = STF::new(img_id, elf, prover_mode);

    c.bench_function("create_proof", |b| {
        b.iter(|| {
            stf.create_recursive_proof::<Prover, Proof, ZKVM>(
                black_box(prev_adapter_proof.clone()),
                black_box(init_account.clone()),
                black_box(new_rollup_proof.clone()),
                black_box(new_rollup_pi.clone()),
                black_box(pubdata_commitments.clone()),
                black_box(versioned_hashes.clone()),
                black_box(nexus_hash),
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, create_proof);
criterion_main!(benches);
