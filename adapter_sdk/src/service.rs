use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter}; // Ensure this is included if you're deserializing JSON bodies.

// Assuming the existence of these modules and types from your project.
use crate::state::AdapterState;
use crate::types::RollupProof;
use nexus_core::traits::{Proof, RollupPublicInputs};

async fn server<PI: RollupPublicInputs, P: Proof<PI>>(
    shared_state: Arc<Mutex<AdapterState<PI, P>>>,
) {
    // Health check route
    let health_check_route = warp::get()
        .and(warp::path("health"))
        .and_then(health_check_handler);

    // Proof handling route
    let proof_route = warp::post()
        .and(warp::path("proof"))
        .and(warp::body::json())
        .and(warp::any().map(move || shared_state.clone()));
    // TODO:  .and_then(handle_proof_handler::<PI, P>);

    // // Combined routes
    // let routes = health_check_route.or(proof_route);

    // // Start the server
    // warp::serve(routes).run(([127, 0, 0, 1], 3031)).await;
}

// Handler for the health check endpoint
async fn health_check_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("OK", StatusCode::OK))
}

// Proof handling function, adapted for generic parameters.
async fn handle_proof_handler<PI: RollupPublicInputs, P: Proof<PI>>(
    proof: RollupProof<PI, P>,
    state: Arc<Mutex<AdapterState<PI, P>>>,
) -> Result<impl Reply, Rejection> {
    // Implement your logic here...
    Ok(warp::reply::with_status("Proof received", StatusCode::OK))
}
