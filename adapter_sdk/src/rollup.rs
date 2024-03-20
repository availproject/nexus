use crate::state::{AdapterState, Proof};
use warp::{http::StatusCode, Filter};

pub async fn server(state: AdapterState) {
    let proof_route = warp::post()
        .and(warp::path("proof"))
        .and(warp::body::json())
        .and_then(handle_proof);

    // Start the warp server
    warp::serve(proof_route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_proof(proof: Proof) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received proof: {:?}", proof);

    Ok(warp::reply::with_status("Proof received", StatusCode::OK))
}
