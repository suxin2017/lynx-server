use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use derive_builder::Builder;
use futures_util::future::join_all;
use http::Extensions;
use http::uri::Authority;
use http_body_util::BodyExt;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::service::TowerToHyperService;
use include_dir::Dir;
use local_ip_address::list_afinet_netifas;
use lynx_db::migration::Migrator;
use rcgen::Certificate;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tower::util::Oneshot;
use tower::{ServiceBuilder, service_fn};
use tracing::{debug, trace, warn};

use crate::client::request_client::RequestClientBuilder;
use crate::common::{HyperReq, is_https_tcp_stream};
use crate::gateway_service::gateway_service_fn;
use crate::layers::error_handle_layer::ErrorHandlerLayer;
use crate::layers::log_layer::LogLayer;
use crate::layers::message_package_layer::message_event_store::MessageEventCache;
use crate::layers::message_package_layer::{MessageEventChannel, RequestMessageEventService};
use crate::layers::req_extension_layer::RequestExtensionLayer;
use crate::layers::trace_id_layer::TraceIdLayer;

pub mod server_ca_manage;
pub mod server_config;

use server_ca_manage::ServerCaManager;
use server_config::ProxyServerConfig;

#[derive(Debug, Clone)]
pub struct StaticDir(pub Dir<'static>);

#[derive(Builder)]
#[builder(build_fn(skip))]
pub struct ProxyServer {
    #[builder(setter(strip_option))]
    pub port: Option<u16>,
    #[builder(setter(skip))]
    pub access_addr_list: Vec<SocketAddr>,
    #[builder(setter(strip_option))]
    pub custom_certs: Option<Arc<Vec<Arc<Certificate>>>>,

    #[builder(setter(strip_option))]
    pub static_dir: Option<Arc<StaticDir>>,

    pub config: Arc<ProxyServerConfig>,

    pub server_ca_manager: Arc<ServerCaManager>,

    pub db_config: ConnectOptions,

    #[builder(setter(skip))]
    pub db_connect: Arc<DatabaseConnection>,
}

impl ProxyServerBuilder {
    pub async fn build(&self) -> Result<ProxyServer> {
        let port = self.port.flatten().unwrap_or(0);
        let network_interfaces = list_afinet_netifas().expect("get network interfaces error");
        let access_addr_list: Vec<SocketAddr> = network_interfaces
            .into_iter()
            .filter(|(_, ip)| ip.is_ipv4())
            .filter(|(_, ip)| ip.is_loopback())
            .map(|(_, ip)| ip)
            .map(|ip| SocketAddr::new(ip, port))
            .collect();
        let custom_certs = self.custom_certs.clone().flatten();
        let db_config = self.db_config.clone().expect("db_config is required");
        let db_con = Database::connect(db_config.clone()).await?;

        Ok(ProxyServer {
            port: self.port.flatten(),
            access_addr_list,
            custom_certs,
            config: self.config.clone().expect("config is required"),
            db_config,
            static_dir: self.static_dir.clone().flatten(),
            server_ca_manager: self
                .server_ca_manager
                .clone()
                .expect("server_ca_manager is required"),
            db_connect: Arc::new(db_con),
        })
    }
}

#[derive(Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub only_localhost: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            only_localhost: false,
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ClientAddr(SocketAddr);

pub trait ClientAddrRequestExt {
    fn get_client_addr(&self) -> Option<ClientAddr>;
}

impl ClientAddrRequestExt for Extensions {
    fn get_client_addr(&self) -> Option<ClientAddr> {
        self.get::<ClientAddr>().cloned()
    }
}

impl ProxyServer {
    pub async fn run(&mut self) -> Result<()> {
        self.bind_tcp_listener_to_hyper().await?;
        Ok(())
    }

    async fn bind_tcp_listener(&self) -> Result<Vec<TcpListener>> {
        let bind_future = self
            .access_addr_list
            .iter()
            .map(|addr| async move {
                let listener = TcpListener::bind(*addr).await?;
                trace!("Server started on: http://{}", listener.local_addr()?);
                trace!("Server started on: https://{}", listener.local_addr()?);
                Ok(listener)
            })
            .collect::<Vec<_>>();
        let tcp_listener = join_all(bind_future).await;

        tcp_listener.into_iter().collect()
    }

    async fn bind_hyper(&self, listener: TcpListener) -> Result<()> {
        let access_addr_list: Arc<Vec<SocketAddr>> = Arc::new(self.access_addr_list.clone());
        let client_custom_certs = self.custom_certs.clone();
        let server_ca_manager = self.server_ca_manager.clone();
        let server_config = self.config.clone();
        let message_event_store = Arc::new(MessageEventCache::default());
        let message_event_cannel = Arc::new(MessageEventChannel::new(message_event_store.clone()));
        let static_dir = self.static_dir.clone();
        let addr_str = listener.local_addr()?.to_string();
        let authority = Authority::from_str(&addr_str)?;
        let self_ca = server_ca_manager.get_server_config(&authority).await?;
        let tls_acceptor = TlsAcceptor::from(self_ca);

        let db_connect = self.db_connect.clone();
        Migrator::up(db_connect.as_ref(), None).await?;

        tokio::spawn(async move {
            loop {
                let (tcp_stream, client_addr) = listener.accept().await.expect("accept failed");

                let tls_acceptor = tls_acceptor.clone();

                let request_client = Arc::new(
                    RequestClientBuilder::default()
                        .custom_certs(client_custom_certs.clone())
                        .build()
                        .expect("build request client error"),
                );

                let server_ca_manager = server_ca_manager.clone();
                let server_config = server_config.clone();
                let message_event_cannel = message_event_cannel.clone();
                let db_connect = db_connect.clone();
                let message_event_store = message_event_store.clone();
                let access_addr_list = access_addr_list.clone();
                let static_dir = static_dir.clone();
                tokio::task::spawn(async move {
                    let svc = service_fn(gateway_service_fn);
                    let svc = ServiceBuilder::new()
                        .layer(RequestExtensionLayer::new(db_connect.clone()))
                        .layer(RequestExtensionLayer::new(request_client))
                        .layer(RequestExtensionLayer::new(ClientAddr(client_addr)))
                        .layer(RequestExtensionLayer::new(server_ca_manager))
                        .layer(RequestExtensionLayer::new(server_config))
                        .layer(RequestExtensionLayer::new(message_event_store))
                        .layer(RequestExtensionLayer::new(message_event_cannel))
                        .layer(RequestExtensionLayer::new(access_addr_list))
                        .layer(RequestExtensionLayer::new(static_dir))
                        .layer(TraceIdLayer)
                        .layer_fn(|inner| RequestMessageEventService { service: inner })
                        .layer(LogLayer)
                        .layer(ErrorHandlerLayer)
                        .service(svc);
                    let transform_svc = service_fn(move |req: HyperReq| {
                        let svc = svc.clone();
                        async move {
                            let req = req.map(|b| b.map_err(|e| anyhow!(e)).boxed());
                            Oneshot::new(svc, req).await
                        }
                    });

                    let svc = TowerToHyperService::new(transform_svc);

                    // TODO： refactor this code let it be more simple
                    if is_https_tcp_stream(&tcp_stream).await {
                        let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                            Ok(tls_stream) => tls_stream,
                            Err(err) => {
                                tracing::error!("failed to perform tls handshake: {:#}", err);
                                return;
                            }
                        };
                        let io = TokioIo::new(tls_stream);
                        let connection = auto::Builder::new(TokioExecutor::new())
                            .serve_connection_with_upgrades(io, svc)
                            .await;
                        if let Err(err) = connection {
                            warn!("Error serving connection: {}", err);
                            debug!("Error serving connection: {:?}", err);
                        }
                    } else {
                        let io = TokioIo::new(tcp_stream);
                        let connection = auto::Builder::new(TokioExecutor::new())
                            .serve_connection_with_upgrades(io, svc)
                            .await;
                        if let Err(err) = connection {
                            warn!("Error serving connection: {}", err);
                            debug!("Error serving connection: {:?}", err);
                        }
                    }
                });
            }
        });
        Ok(())
    }

    async fn bind_tcp_listener_to_hyper(&mut self) -> Result<()> {
        let tcp_listeners = self.bind_tcp_listener().await?;
        let bind_addrs: Vec<SocketAddr> = tcp_listeners
            .iter()
            .filter_map(|listener| listener.local_addr().ok())
            .collect();
        self.access_addr_list = bind_addrs;
        for tcp_listener in tcp_listeners {
            self.bind_hyper(tcp_listener).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::{
        server_ca_manage::ServerCaManagerBuilder, server_config::ProxyServerConfigBuilder, *,
    };

    async fn build_test_proxy_server() -> Result<ProxyServer> {
        let fixed_temp_dir_path = TempDir::new("proxy_test")?;
        let fixed_temp_dir_path = fixed_temp_dir_path.path();

        let server_config = ProxyServerConfigBuilder::default()
            .root_cert_file_path(fixed_temp_dir_path.join("root.pem"))
            .root_key_file_path(fixed_temp_dir_path.join("key.pem"))
            .build()?;

        let server_ca_manager = ServerCaManagerBuilder::new(
            server_config.root_cert_file_path.clone(),
            server_config.root_key_file_path.clone(),
        )
        .build()?;

        let proxy_server = ProxyServerBuilder::default()
            .config(Arc::new(server_config))
            .server_ca_manager(Arc::new(server_ca_manager))
            .db_config(ConnectOptions::new("sqlite::memory:"))
            .build()
            .await?;
        Ok(proxy_server)
    }

    #[tokio::test]
    async fn build_test() -> Result<()> {
        build_test_proxy_server().await?;
        Ok(())
    }

    #[tokio::test]
    async fn listener_test() -> Result<()> {
        let server = build_test_proxy_server().await?;

        let data = server.bind_tcp_listener().await?;

        for result in data {
            let addr = result.local_addr()?;
            println!("Tcp started on: {}", addr);
        }
        Ok(())
    }

    #[tokio::test]
    async fn hyper_test() -> Result<()> {
        let server = build_test_proxy_server().await?;

        let tcp_listeners = server.bind_tcp_listener().await?;

        for tcp_listener in tcp_listeners {
            server.bind_hyper(tcp_listener).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn run_test() -> Result<()> {
        let mut server = build_test_proxy_server().await?;

        server.run().await?;
        Ok(())
    }
}
