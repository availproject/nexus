use crate::traits::{Proof, RollupPublicInputs};
use crate::{state::AdapterState, types::RollupProof};
use avail_core::DataProof;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter};

async fn health_check_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("OK", StatusCode::OK))
}

async fn handle_proof_handler<
    PI: RollupPublicInputs + Clone + Serialize + DeserializeOwned + Send,
    P: Proof<PI> + Clone + Serialize + DeserializeOwned + Send,
>(
    state: Arc<Mutex<AdapterState<PI, P>>>,
    proof: RollupProof<PI, P>,
) -> Result<impl Reply, Rejection> {
    let mut locked_state = state.lock().await;

    locked_state.add_proof(proof).await;

    Ok(warp::reply::with_status("Proof received", StatusCode::OK))
}

async fn handle_blob_handler<
    PI: RollupPublicInputs + Clone + Serialize + DeserializeOwned + Send,
    P: Proof<PI> + Clone + Serialize + DeserializeOwned + Send,
>(
    state: Arc<Mutex<AdapterState<PI, P>>>,
    data_proof: DataProof,
) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status(
        "Data Blob with inclusion proof received",
        StatusCode::OK,
    ))
}

pub async fn server<
    I: RollupPublicInputs + Clone + Send + Sync + 'static + DeserializeOwned + Serialize,
    P: Proof<I> + Send + Clone + Sync + 'static + DeserializeOwned + Serialize,
>(
    state: Arc<Mutex<AdapterState<I, P>>>,
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
