use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use lynx_core::proxy_server::{
    ProxyServerBuilder, server_ca_manage::ServerCaManagerBuilder,
    server_config::ProxyServerConfigBuilder,
};
use opentelemetry::trace::TraceError;
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::Sampler;
use sea_orm::ConnectOptions;
use tokio::signal;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

/// 初始化 Jaeger 追踪器
fn init_jaeger_tracer() -> Result<(), TraceError> {
    // 使用 OTLP 导出器连接到 Jaeger
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint("http://127.0.0.1:4318");

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "lynx-proxy-server"),
                    KeyValue::new("service.version", "1.0.0"),
                    KeyValue::new("service.component", "proxy-server"),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // 初始化 tracing 订阅者，结合 Jaeger 和控制台输出
    let telemetry_layer = OpenTelemetryLayer::new(tracer);

    Registry::default()
        .with(EnvFilter::new("lynx_core=trace,hyper=trace"))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_line_number(true),
        )
        .with(telemetry_layer)
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_jaeger_tracer()?;

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
    // 关闭追踪器
    global::shutdown_tracer_provider();
    Ok(())
}
