use nexus_core::db::NodeDB;
use nexus_core::types::AppId;

pub struct Adapter {
    app_id: AppId,
    db: NodeDB,
}

pub struct Config<'a> {
    proof_type: ProofType,
    app_id: AppId,
    db_path: &'a str,
}

enum ProofType {
    Groth16,
    RiscZero,
}
