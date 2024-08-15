use crate::traits::RollupProof;
use crate::{state::AdapterState, types::RollupProofWithPublicInputs};
use nexus_core::types::Proof;
use nexus_core::zkvm::traits::{ZKVMEnv, ZKVMProof};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug as DebugTrait;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter};

async fn health_check_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("OK", StatusCode::OK))
}

async fn handle_proof_handler<
    P: RollupProof + Clone + Serialize + DeserializeOwned + Send,
    Z: ZKVMEnv,
    ZP: ZKVMProof + DebugTrait + Clone + Serialize + DeserializeOwned + Send + TryInto<Proof>,
>(
    state: Arc<Mutex<AdapterState<P, Z, ZP>>>,
    proof: RollupProofWithPublicInputs<P>,
) -> Result<impl Reply, Rejection>
where
    <ZP as TryInto<Proof>>::Error: Into<anyhow::Error>,
{
    let mut locked_state = state.lock().await;

    locked_state.add_proof(proof).await;

    Ok(warp::reply::with_status("Proof received", StatusCode::OK))
}

pub async fn server<
    P: RollupProof + Send + Clone + Sync + DeserializeOwned + Serialize,
    Z: ZKVMEnv,
    ZP: ZKVMProof + DebugTrait + Clone + Serialize + DeserializeOwned + Send + TryInto<Proof>,
>(
    state: Arc<Mutex<AdapterState<P, Z, ZP>>>,
) where
    <ZP as TryInto<Proof>>::Error: Into<anyhow::Error>,
{
    // Health check route
    let health_check_route = warp::get()
        .and(warp::path("health"))
        .and_then(health_check_handler);

    // Proof handling route
    let proof_route = warp::post()
        .and(warp::path("proof"))
        .and(warp::any().map(move || state.clone()))
        .and(warp::body::json())
        .and_then(handle_proof_handler);

    // Combined routes
    let routes = health_check_route.or(proof_route);

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 3031)).await;
}
