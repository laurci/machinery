mod user;

mod api;
machinery::load_services!();

use machinery::{machinery, MachineryStandaloneConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let listen_addr = std::net::SocketAddr::from(([0, 0, 0, 0], 9797));

    machinery!()
        .with_standalone_config(MachineryStandaloneConfig { listen_addr })
        .boot()
        .await
}
