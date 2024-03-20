pub mod adapter;
pub mod rollup;
pub mod state;
pub mod types;

use rollup::server;
use tokio;

#[tokio::main]
async fn main() {
    server().await;
}
