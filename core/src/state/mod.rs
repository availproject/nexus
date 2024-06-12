#[cfg(any(feature = "native"))]
mod merkle_store;

#[cfg(any(feature = "native"))]
pub mod vm_state;
#[cfg(any(feature = "native"))]
pub use self::merkle_store::*;
#[cfg(any(feature = "native"))]
pub use self::vm_state::VmState;
#[cfg(any(feature = "native"))]
pub use sparse_merkle_tree;

pub mod types;