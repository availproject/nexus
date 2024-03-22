// use std::sync::Arc;

// use crate::{
//     adapter_zkvm::verify_proof,
//     proof_storage::{GenericProof, ProofTrait},
//     state::AdapterState,
// };
// use serde::Deserialize;
// use tokio::sync::Mutex;
// use warp::{http::StatusCode, Filter};

// pub async fn server<P: ProofTrait + 'static + Send + Sync>(
//     shared_state: Arc<Mutex<AdapterState<P>>>,
// ) {
//     let proof_route = warp::post()
//         .and(warp::path("proof"))
//         .and(warp::body::json())
//         .and(with_shared_state(Arc::clone(&shared_state)))
//         .and_then(handle_proof);

//     // Start the warp server
//     warp::serve(proof_route).run(([127, 0, 0, 1], 3030)).await;
// }

// async fn handle_proof<P>(
//     proof: GenericProof<P>,
//     state: Arc<Mutex<AdapterState<P>>>,
// ) -> Result<impl warp::Reply, warp::Rejection>
// where
//     P: ProofTrait + 'static + Send + Sync,
// {
//     println!("Received proof: {:?}", proof);

//     Ok(warp::reply::with_status("Proof received", StatusCode::OK))
// }

// fn with_shared_state<P: ProofTrait + 'static + Send + Sync>(
//     shared_state: Arc<Mutex<AdapterState<P>>>,
// ) -> impl Filter<Extract = (Arc<Mutex<AdapterState<P>>>,), Error = std::convert::Infallible> + Clone
// {
//     warp::any().map(move || shared_state.clone())
// }
