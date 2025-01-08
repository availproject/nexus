use std::fmt::Debug;

use adapter_sdk::{state::AdapterState, types::AdapterConfig};
use demo_rollup_core::DemoProof;
use methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::{
    types::{AppId, StatementDigest},
    zkvm::{
        risczero::{RiscZeroProof, ZKVM},
        traits::{ZKVMEnv, ZKVMProof, ZKVMProver},
        ProverMode,
    },
};

fn main() {
    //! TODO: we can it to configure it for sp1 as well
    let mut adapter: AdapterState<DemoProof, ZKVM, RiscZeroProof> = AdapterState::new(
        &String::from("adapter_store"),
        AdapterConfig {
            app_id: AppId(100),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
            prover_mode: ProverMode::MockProof,
            //TODO: Replace with configurable value.
            avail_url: String::from("wss://turing-rpc.avail.so:443/ws"),
        },
    );
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(adapter.run());
}
