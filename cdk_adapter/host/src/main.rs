use adapter_sdk::{state::AdapterState, types::AdapterConfig, service::server};
use cdk_adapter_core::CdkProof;
use cdk_adapter_methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::types::{AppId, StatementDigest};
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    let mut adapter: AdapterState<CdkProof> = AdapterState::new(
        String::from("adapter_store"),
        AdapterConfig {
            app_id: AppId(100),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
        },
    );
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(server(Arc::new(Mutex::new(adapter))));
}
