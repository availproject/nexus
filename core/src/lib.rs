#[cfg(any(feature = "native"))]
pub mod db;
//mod new_stf;
#[cfg(any(feature = "native"))]
mod state;
// #[cfg(any(feature = "native"))]
// pub mod state_machine;
mod traits;
//mod trie;
pub mod agg_types;
#[cfg(any(feature = "native"))]
pub mod mempool;
#[cfg(any(feature = "native"))]
pub mod simple_state_machine;
pub mod simple_stf;
pub mod types;
//pub mod zkvm_state_machine;
pub mod simple_zkvm_state_machine;
