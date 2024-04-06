use adapter_sdk::{
    adapter_zkvm::verify_proof,
    state::AdapterState,
    types::{AdapterConfig, AdapterPrivateInputs, AdapterPublicInputs},
};
use anyhow::{anyhow, Error};

use demo_rollup_core::{DemoProof, DemoRollupPublicInputs};
use methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::types::{AppId, StatementDigest};
use risc0_zkvm::{default_prover, ExecutorEnv, InnerReceipt};

const APP_ID: AppId = AppId(1);

fn main() {
    let mut adapter: AdapterState<DemoRollupPublicInputs, DemoProof> = AdapterState::new(
        String::from("adapter_store"),
        AdapterConfig {
            app_id: APP_ID,
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
        },
    );
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(adapter.run());
}
