#[cfg(any(feature = "native"))]
pub mod db;
mod new_stf;
#[cfg(any(feature = "native"))]
mod state;
#[cfg(any(feature = "native"))]
pub mod state_machine;
mod traits;
//mod trie;
#[cfg(any(feature = "native"))]
pub mod mempool;
pub mod types;
pub mod zkvm_state_machine;
