use std::sync::Arc;

use anyhow::{Result, anyhow};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use reqwest::{self, Certificate, Client, Response};
use reqwest_websocket::{CloseCode, Message, RequestBuilderExt, WebSocket};
use tokio::spawn;
use tracing::{debug, trace};

use crate::server::MockServer;

pub struct MockClientInner {
    pub direct_client: Arc<reqwest::Client>,
    pub proxy_client: Arc<reqwest::Client>,
}

impl MockClientInner {
    pub async fn get(&self, url: &str) -> (Result<Response>, Result<Response>) {
        trace!("send request {url}");
        (
            self.direct_client
                .get(url)
                .send()
                .await
                .map_err(|e| anyhow!(e)),
            self.proxy_client
                .get(url)
                .send()
                .await
                .map_err(|e| anyhow!(e)),
        )
    }

    pub async fn ws(
        &self,
        url: &str,
    ) -> (
        Result<reqwest_websocket::UpgradeResponse, reqwest_websocket::Error>,
        Result<reqwest_websocket::UpgradeResponse, reqwest_websocket::Error>,
    ) {
        (
            self.direct_client.get(url).upgrade().send().await,
            self.proxy_client.get(url).upgrade().send().await,
        )
    }
}
pub struct MockClient(pub Arc<MockClientInner>);

#[derive(Debug, Clone)]
pub struct MessageWrapper(pub Message);

impl PartialEq for MessageWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Message::Text(l0), Message::Text(r0)) => l0 == r0,
            (Message::Binary(l0), Message::Binary(r0)) => l0 == r0,
            (Message::Ping(l0), Message::Ping(r0)) => l0 == r0,
            (Message::Pong(l0), Message::Pong(r0)) => l0 == r0,
            (
                Message::Close {
                    code: l_code,
                    reason: l_reason,
                },
                Message::Close {
                    code: r_code,
                    reason: r_reason,
                },
            ) => l_code == r_code && l_reason == r_reason,
            _ => false,
        }
    }
}

impl MockClient {
    pub fn get_request_client(&self) -> Arc<Client> {
        self.0.direct_client.clone()
    }
    pub fn new(
        custom_cert: Option<Vec<Arc<rcgen::Certificate>>>,
        proxy_url: Option<String>,
    ) -> Result<Self> {
        let direct_client = Self::build_client(&custom_cert, None)?;
        if let Some(proxy_url) = &proxy_url {
            trace!("proxy addr: {proxy_url}");
        }
        let proxy_client = Self::build_client(&custom_cert, proxy_url.clone())?;
        Ok(MockClient(Arc::new(MockClientInner {
            direct_client: Arc::new(direct_client),
            proxy_client: Arc::new(proxy_client),
        })))
    }

    fn build_client(
        custom_cert: &Option<Vec<Arc<rcgen::Certificate>>>,
        proxy_url: Option<String>,
    ) -> Result<Client> {
        let certs = custom_cert.as_ref().map(|certs| {
            certs
                .iter()
                .map(|cert| reqwest::Certificate::from_pem(cert.pem().as_bytes()).unwrap())
                .collect::<Vec<Certificate>>()
        });

        let mut client = Client::builder().use_rustls_tls();

        if let Some(certs) = certs {
            for cert in certs {
                client = client.add_root_certificate(cert);
            }
        }
        client = client.no_proxy();
        if let Some(proxy_url) = proxy_url {
            client = client.proxy(reqwest::Proxy::all(proxy_url).unwrap());
        }
        client.build().map_err(|e| anyhow!(e))
    }

    pub async fn get(&self, url: &str) -> (Result<Response>, Result<Response>) {
        self.0.get(url).await
    }

    pub async fn test_request_http_request(&self, server: &MockServer) -> Result<()> {
        for path in server.get_http_mock_paths() {
            let (direct_res, proxy_res) = self.get(path.as_str()).await;
            Self::assert_equality_res(direct_res, proxy_res).await?;
        }
        Ok(())
    }

    pub async fn test_request_https_request(&self, server: &MockServer) -> Result<()> {
        for path in server.get_https_mock_paths() {
            let (direct_res, proxy_res) = self.get(path.as_str()).await;
            Self::assert_equality_res(direct_res, proxy_res).await?;
        }
        Ok(())
    }

    pub async fn test_request_websocket(&self, server: &MockServer) -> Result<()> {
        let ws_path = server.get_websocket_path();
        let (direct_res, proxy_res) = self.ws(ws_path.as_str()).await;
        Self::assert_equality_ws(direct_res, proxy_res).await?;
        Ok(())
    }

    pub async fn test_real_world_http_request(&self) -> Result<()> {
        let path = "http://example.com/";
        let (direct_res, proxy_res) = self.get(path).await;
        Self::assert_equality_res(direct_res, proxy_res).await?;
        Ok(())
    }

    pub async fn test_real_world_https_request(&self) -> Result<()> {
        let path = "https://example.com/";
        let (direct_res, proxy_res) = self.get(path).await;
        Self::assert_equality_res(direct_res, proxy_res).await?;
        Ok(())
    }

    pub async fn test_real_world_websocket_request(&self) -> Result<()> {
        let ws_path = "ws://echo.websocket.org";
        let (direct_res, proxy_res) = self.ws(ws_path).await;
        Self::assert_equality_ws(direct_res, proxy_res).await?;
        Ok(())
    }

    pub async fn test_real_world_tls_websocket_request(&self) -> Result<()> {
        let ws_path = "wss://echo.websocket.org";
        let (direct_res, proxy_res) = self.ws(ws_path).await;
        Self::assert_equality_ws(direct_res, proxy_res).await?;
        Ok(())
    }

    pub async fn test_request_tls_websocket(&self, server: &MockServer) -> Result<()> {
        let ws_path = server.get_tls_websocket_path();
        let (direct_res, proxy_res) = self.ws(ws_path.as_str()).await;
        Self::assert_equality_ws(direct_res, proxy_res).await?;
        Ok(())
    }

    async fn ws(
        &self,
        ws_path: &str,
    ) -> (
        Result<reqwest_websocket::UpgradeResponse, reqwest_websocket::Error>,
        Result<reqwest_websocket::UpgradeResponse, reqwest_websocket::Error>,
    ) {
        self.0.ws(ws_path).await
    }

    async fn assert_equality_res(
        direct_res: Result<Response>,
        proxy_res: Result<Response>,
    ) -> Result<()> {
        debug!("direct res: {:?}", direct_res);
        debug!("proxy res: {:?}", proxy_res);
        assert_eq!(direct_res.is_ok(), proxy_res.is_ok());
        if let (Ok(direct_res), Ok(proxy_res)) = (direct_res, proxy_res) {
            assert_eq!(direct_res.status(), proxy_res.status());
            assert_eq!(direct_res.bytes().await?, proxy_res.bytes().await?);
        }
        Ok(())
    }

    async fn assert_equality_ws(
        direct_res: Result<reqwest_websocket::UpgradeResponse, reqwest_websocket::Error>,
        proxy_res: Result<reqwest_websocket::UpgradeResponse, reqwest_websocket::Error>,
    ) -> Result<()> {
        assert_eq!(direct_res.is_ok(), proxy_res.is_ok());
        if let (Ok(direct_res), Ok(proxy_res)) = (direct_res, proxy_res) {
            assert_eq!(direct_res.status(), proxy_res.status());
            let (dw, pw) = (
                direct_res.into_websocket().await?,
                proxy_res.into_websocket().await?,
            );
            let (dsi, dst) = dw.split();
            let (psi, pst) = pw.split();
            let a = spawn(async move { send_message(dsi, dst).await });

            let b = spawn(async move { send_message(psi, pst).await });

            let d_recv = a.await??;
            let p_recv = b.await??;

            debug!("direct recv: {:?}", d_recv);
            debug!("proxy recv: {:?}", p_recv);

            assert_eq!(d_recv.len(), p_recv.len());
            for (d_msg, p_msg) in d_recv.into_iter().zip(p_recv.into_iter()) {
                assert_eq!(MessageWrapper(d_msg?), MessageWrapper(p_msg?));
            }
        }
        Ok(())
    }
}

async fn send_message(
    mut si: SplitSink<reqwest_websocket::WebSocket, Message>,
    mut st: SplitStream<WebSocket>,
) -> Result<Vec<Result<Message, reqwest_websocket::Error>>> {
    let mut res = Vec::new();
    si.send(Message::Text("Hello".into())).await?;
    let msg = st.next().await;
    res.push(msg);
    si.send(Message::Binary(b"World".into())).await?;
    let msg = st.next().await;
    res.push(msg);
    si.send(Message::Close {
        code: CloseCode::Normal,
        reason: "".into(),
    })
    .await?;

    Ok(res
        .into_iter()
        .filter_map(|m| {
            if let Some(Ok(m)) = m {
                Some(Ok(m))
            } else {
                None
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_client() {
        let mock_client = MockClient::new(None, Some("http://example.proxy/".into()));
        assert!(mock_client.is_ok());
    }

    #[tokio::test]
    async fn test_get_url_with_proxy() {
        let mock_client = MockClient::new(None, None);
        assert!(mock_client.is_ok());
    }
}
