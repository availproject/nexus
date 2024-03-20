pub mod rollup;
pub mod state;
pub mod types;

use rollup::server;

#[tokio::main]
async fn main() {
    server().await;
}
