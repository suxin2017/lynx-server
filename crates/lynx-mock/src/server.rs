use crate::mark_service::MarkService;
use crate::mock_server_fn::{HTTP_PATH_LIST, MockPath, WS_PATH, mock_server_fn};
use anyhow::Result;
use hyper_util::rt::{TokioExecutor, TokioIo};
use lynx_cert::{gen_server_config_by_ca, get_self_signed_cert, read_cert_and_key_by_file};
use rcgen::{Certificate, KeyPair};
use std::fs;
use std::path::PathBuf;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};
use tower::ServiceBuilder;
use tracing::info;

pub async fn is_https_tcp_stream(tcp_stream: &tokio::net::TcpStream) -> bool {
    let mut buf = [0; 1];
    match tcp_stream.peek(&mut buf).await {
        Ok(n) => n == 1 && buf[0] == 0x16,
        Err(_) => false,
    }
}

pub fn mock_server_config(
    cert_path: &PathBuf,
    key_path: &PathBuf,
) -> Result<(ServerConfig, Arc<Certificate>, KeyPair)> {
    let (cert, key) = if cert_path.exists() && key_path.exists() {
        let (cert, key) = read_cert_and_key_by_file(key_path, cert_path)?;
        let cert = Arc::new(cert);
        (cert, key)
    } else {
        let (cert, key) = get_self_signed_cert(None)?;
        let cert = Arc::new(cert);
        (cert, key)
    };
    let server_config = gen_server_config_by_ca(&[cert.clone()], &key)?;
    Ok((server_config, cert, key))
}

pub struct MockServer {
    pub addr: SocketAddr,
    pub cert: Arc<Certificate>,
    key: KeyPair,
    server_config: Arc<ServerConfig>,
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl MockServer {
    pub fn new(port: Option<u16>) -> Self {
        let addr = Self::init_addr(port);
        let fixed_temp_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/temp");

        if !fixed_temp_dir_path.exists() {
            fs::create_dir_all(&fixed_temp_dir_path).expect("Failed to create temp dir");
        }

        let cert_path = fixed_temp_dir_path.join("cert.pem");
        let key_path = fixed_temp_dir_path.join("key.pem");
        // Build TLS configuration.
        let (server_config, cert, key) =
            mock_server_config(&cert_path, &key_path).expect("Failed to generate server config");
        MockServer {
            addr,
            server_config: Arc::new(server_config),
            cert,
            key,
            cert_path,
            key_path,
        }
    }

    fn init_addr(port: Option<u16>) -> SocketAddr {
        if let Some(port) = port {
            SocketAddr::from(([127, 0, 0, 1], port))
        } else {
            SocketAddr::from(([127, 0, 0, 1], 0))
        }
    }

    pub fn get_http_mock_paths(&self) -> Vec<String> {
        HTTP_PATH_LIST
            .iter()
            .filter(|path| !matches!(path, MockPath::Slow | MockPath::Timeout))
            .map(|path| format!("http://{}{}", self.addr, path))
            .collect()
    }

    pub fn get_https_mock_paths(&self) -> Vec<String> {
        HTTP_PATH_LIST
            .iter()
            .filter(|path| !matches!(path, MockPath::Slow | MockPath::Timeout))
            .map(|path| format!("https://{}{}", self.addr, path))
            .collect()
    }

    pub fn get_websocket_path(&self) -> String {
        format!("ws://{}{}", self.addr, WS_PATH)
    }

    pub fn get_tls_websocket_path(&self) -> String {
        format!("wss://{}{}", self.addr, WS_PATH)
    }

    pub fn write_cert_to_file(&mut self) -> Result<()> {
        if self.cert_path.exists() && self.key_path.exists() {
            return Ok(());
        }
        fs::write(&self.cert_path, self.cert.pem().as_bytes())?;
        fs::write(&self.key_path, self.key.serialize_pem().as_bytes())?;
        Ok(())
    }

    pub async fn start_server(&mut self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        let addr = listener.local_addr()?;
        self.addr = addr;
        info!("Listening on \n    http://{} \n    https://{}", addr, addr);
        println!("Listening on \n    http://{} \n    https://{}", addr, addr);

        let tls_acceptor = TlsAcceptor::from(self.server_config.clone());
        let addr_mark = Arc::new(format!("{}", self.addr));

        tokio::spawn(async move {
            loop {
                let (tcp_stream, _) = listener.accept().await.unwrap();
                let addr_mark = addr_mark.clone();
                let tls_acceptor = tls_acceptor.clone();
                tokio::task::spawn(async move {
                    let svc = hyper::service::service_fn(mock_server_fn);
                    let svc = ServiceBuilder::new()
                        .layer_fn(|s| MarkService::new(s, addr_mark.clone()))
                        .service(svc);
                    if is_https_tcp_stream(&tcp_stream).await {
                        let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                            Ok(tls_stream) => tls_stream,
                            Err(err) => {
                                eprintln!("failed to perform tls handshake: {:#}", err);
                                return;
                            }
                        };
                        hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                            .serve_connection_with_upgrades(TokioIo::new(tls_stream), svc)
                            .await
                            .unwrap();
                    } else {
                        hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                            .serve_connection_with_upgrades(TokioIo::new(tcp_stream), svc)
                            .await
                            .unwrap();
                    }
                });
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn start_server() {
        let mut server = MockServer::new(Some(3000));
        let res = server.start_server().await;

        assert!(res.is_ok());
    }
}
