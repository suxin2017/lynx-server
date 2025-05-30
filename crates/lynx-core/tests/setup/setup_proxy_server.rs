use std::{fs, path::PathBuf, sync::Arc};

use anyhow::{Ok, Result};
use lynx_core::proxy_server::{
    ProxyServer, ProxyServerBuilder, server_ca_manage::ServerCaManagerBuilder,
    server_config::ProxyServerConfigBuilder,
};
use rcgen::Certificate;
use sea_orm::ConnectOptions;

pub async fn setup_proxy_server(
    custom_certs: Option<Arc<Vec<Arc<Certificate>>>>,
) -> Result<ProxyServer> {
    let fixed_temp_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/temp");

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

    let mut proxy_server_builder = ProxyServerBuilder::default();

    proxy_server_builder
        .config(Arc::new(server_config))
        .server_ca_manager(Arc::new(server_ca_manager))
        .db_config(ConnectOptions::new("sqlite::memory:"));
    if let Some(custom_certs) = custom_certs {
        proxy_server_builder.custom_certs(custom_certs);
    }

    let mut proxy_server = proxy_server_builder.build().await?;
    proxy_server.run().await?;
    Ok(proxy_server)
}
