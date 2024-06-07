use crate::traits::Proof;
use crate::{state::AdapterState, types::RollupProof};
use anyhow::{anyhow, Error};
use nexus_core::types::H256;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject::Rejection, reply::Reply, Filter};
use warp::filters::log::custom;
use env_logger;
use tracing::info;
use tracing_subscriber::fmt::Subscriber;

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
    println!("Received Proof!");

    Ok(warp::reply::with_status("Proof received", StatusCode::OK))
}

async fn handle_submit_blob<P: Proof + Clone + Serialize + DeserializeOwned + Send>(
    state: Arc<Mutex<AdapterState<P>>>,
    request: BlobDataRequest,
) -> Result<impl Reply, Rejection> {
    let mut locked_state = state.lock().await;
    let blob = &request.blob[..];
    locked_state.store_blob(blob).await;
    Ok(warp::reply::with_status(
        "Data Blob received and posted on Avail DA",
        StatusCode::OK,
    ))
}

pub async fn server<P: Proof + Send + Clone + Sync + 'static + DeserializeOwned + Serialize>(
    state: Arc<Mutex<AdapterState<P>>>,
) {
    tracing_subscriber::fmt::init();

    let custom_logger = custom(|info| {
        info!(
            target: "warp_server",
            method = %info.method(),
            path = %info.path(),
            status = info.status().as_u16(),
            "request logged"
        );
    });

    // Health check route
    let health_check_route = warp::get()
        .and(warp::path("health"))
        .and_then(health_check_handler)
        .with(custom_logger.clone());

    let state_for_proof_route = state.clone();
    // Proof handling route
    let proof_route = warp::post()
        .and(warp::path("proof"))
        .and(warp::any().map(move || state_for_proof_route.clone()))
        .and(warp::body::json())
        .and_then(handle_proof_handler)
        .with(custom_logger.clone());

    let blob_data_route = warp::post()
        .and(warp::path("blob_data"))
        .and(warp::any().map(move || state.clone()))
        .and(warp::body::json())
        .and_then(handle_submit_blob)
        .with(custom_logger.clone());

    // // Combined routes
    let routes = health_check_route.or(proof_route).or(blob_data_route);

    // Start the server
    warp::serve(routes).run(([0, 0, 0, 0], 3031)).await;
}
