mod api;
machinery::load_services!();

use machinery::{machinery, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 9797));
    machinery!().listen(&addr).await
}
