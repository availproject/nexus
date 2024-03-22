use core::convert::Infallible;
use nexus_core::agg_types::SubmitProofTransaction;
use std::sync::{Arc, Mutex};
use warp::{http::StatusCode, reply::Reply, Filter, Rejection};

pub fn routes(
    proofs: Arc<Mutex<Vec<SubmitProofTransaction>>>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let receive_proof = warp::post()
        .and(warp::path("tx"))
        .and(warp::body::json())
        .and(with_proofs(proofs))
        .and_then(store_tx);

    receive_proof
}

// This function clones the Arc to move into the async block
fn with_proofs(
    proofs: Arc<Mutex<Vec<SubmitProofTransaction>>>,
) -> impl Filter<
    Extract = (Arc<Mutex<Vec<SubmitProofTransaction>>>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || proofs.clone())
}

pub async fn store_tx(
    proofs: Arc<Mutex<Vec<SubmitProofTransaction>>>,
    tx: SubmitProofTransaction,
) -> Result<impl Reply, Infallible> {
    let mut proofs = proofs.lock().unwrap(); // Acquire the lock
    proofs.push(tx.into()); // Assuming SubmitProofTransaction can be converted into SubmitProofTransaction

    Ok(warp::reply::with_status("Added tx", StatusCode::OK))
}
