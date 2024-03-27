use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter}; // Ensure this is included if you're deserializing JSON bodies.

// Assuming the existence of these modules and types from your project.
use crate::state::AdapterState;
use crate::types::RollupProof;
use nexus_core::traits::{Proof, RollupPublicInputs};
use std::marker::{Send, Sync};

// Handler for the health check endpoint
async fn health_check_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("OK", StatusCode::OK))
}

// Proof handling function, adapted for generic parameters.
async fn handle_proof_handler<PI: RollupPublicInputs, P: Proof<PI>>(
    state: Arc<Mutex<AdapterState<PI, P>>>,
    proof: RollupProof<PI, P>,
) -> Result<impl Reply, Rejection> {
    let mut locked_state = state.lock().await;

    locked_state.add_proof(proof);
    // Implement your logic here...
    Ok(warp::reply::with_status("Proof received", StatusCode::OK))
}

async fn demo<I: RollupPublicInputs + Send + Sync, P: Proof<I> + Send + Sync>(
    state: Arc<Mutex<AdapterState<I, P>>>,
    proof: String,
) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("Proof received", StatusCode::OK))
}

async fn server<
    I: RollupPublicInputs + Send + Sync + 'static,
    P: Proof<I> + Send + Sync + 'static,
>(
    state: Arc<Mutex<AdapterState<I, P>>>,
) {
    // Health check route
    let health_check_route = warp::get()
        .and(warp::path("health"))
        .and_then(health_check_handler);

    // Proof handling route
    let proof_route = warp::post()
        .and(warp::path("proof"))
        .and(warp::any().map(move || state.clone()))
        .and(warp::body::json())
        .and_then(demo);

    // // Combined routes
    let routes = health_check_route.or(proof_route);

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 3031)).await;
}
