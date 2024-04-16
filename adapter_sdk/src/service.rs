use crate::traits::Proof;
use crate::{state::AdapterState, types::RollupProof};
use anyhow::{anyhow, Error};
use nexus_core::types::H256;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter};

#[derive(Debug, Deserialize, Serialize)]
struct BlobDataRequest {
    blob: Vec<u8>,
}

async fn health_check_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("OK", StatusCode::OK))
}

async fn handle_proof_handler<P: Proof + Clone + Serialize + DeserializeOwned + Send>(
    state: Arc<Mutex<AdapterState<P>>>,
    proof: RollupProof<P>,
) -> Result<impl Reply, Rejection> {
    let mut locked_state = state.lock().await;

    locked_state.add_proof(proof).await;

    Ok(warp::reply::with_status("Proof received", StatusCode::OK))
}

async fn handle_blob_handler<P: Proof + Clone + Serialize + DeserializeOwned + Send>(
    state: Arc<Mutex<AdapterState<P>>>,
    request: BlobDataRequest,
) -> Result<impl Reply, Rejection> {
    let mut locked_state = state.lock().await;
    let blob = &request.blob[..];
    locked_state.store_blob(blob).await;
    Ok(warp::reply::with_status(
        "Data Blob with inclusion proof received",
        StatusCode::OK,
    ))
}

pub async fn server<P: Proof + Send + Clone + Sync + 'static + DeserializeOwned + Serialize>(
    state: Arc<Mutex<AdapterState<P>>>,
) {
    // Health check route
    let health_check_route = warp::get()
        .and(warp::path("health"))
        .and_then(health_check_handler);

    let state_for_proof_route = state.clone();
    // Proof handling route
    let proof_route = warp::post()
        .and(warp::path("proof"))
        .and(warp::any().map(move || state_for_proof_route.clone()))
        .and(warp::body::json())
        .and_then(handle_proof_handler);

    let blob_data_route = warp::post()
        .and(warp::path("blob_data"))
        .and(warp::any().map(move || state.clone()))
        .and(warp::body::json())
        .and_then(handle_blob_handler);

    // // Combined routes
    let routes = health_check_route.or(proof_route).or(blob_data_route);

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 3031)).await;
}
