use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use lynx_core::proxy_server::{
    ProxyServerBuilder, server_ca_manage::ServerCaManagerBuilder,
    server_config::ProxyServerConfigBuilder,
};
use sea_orm::ConnectOptions;
use tokio::signal;
use tokio_rustls::rustls;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider using ring - must be done before any TLS operations
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Failed to install rustls crypto provider"))?;

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("lynx_core=trace".parse()?))
        .init();

    let fixed_temp_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/temp");

    if !fixed_temp_dir_path.exists() {
        fs::create_dir_all(&fixed_temp_dir_path)?;
    }

    let server_config = ProxyServerConfigBuilder::default()
        .root_cert_file_path(fixed_temp_dir_path.join("root.pem"))
        .root_key_file_path(fixed_temp_dir_path.join("key.pem"))
        .build()?;

    let server_ca_manager = ServerCaManagerBuilder::new(
        server_config.root_cert_file_path.clone(),
        server_config.root_key_file_path.clone(),
    )
    .build()?;

    let mut proxy_server = ProxyServerBuilder::default()
        .config(Arc::new(server_config))
        .port(3000)
        .server_ca_manager(Arc::new(server_ca_manager))
        .db_config(ConnectOptions::new(format!(
            "sqlite://{}/lynx.db?mode=rwc",
            fixed_temp_dir_path.to_string_lossy()
        )))
        .build()
        .await?;
    proxy_server.run().await?;

    signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C signal handler");
    Ok(())
}
