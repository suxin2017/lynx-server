use anyhow::{Result, anyhow};
use async_compression::tokio::bufread::GzipEncoder;
use bytes::Bytes;
use futures_util::{SinkExt, TryStreamExt};
use http::{
    Method, StatusCode,
    header::{CONTENT_ENCODING, CONTENT_TYPE},
};
use http_body_util::{BodyExt, Full, StreamBody, combinators::BoxBody};
use hyper::{
    Request, Response,
    body::{Frame, Incoming},
};
use hyper_tungstenite::tungstenite::Message;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tracing::{instrument, trace, warn};
use serde_json;

use std::{fmt::Display, time::Duration};
use tokio::sync::broadcast;
use tokio::time::interval;
use tokio_util::io::ReaderStream;

pub const HELLO_WORLD: &str = "Hello, World!";

pub const HELLO_PATH: &str = "/hello";
pub const GZIP_PATH: &str = "/gzip";
pub const ECHO_PATH: &str = "/echo";
pub const WEBSOCKET_PATH: &str = "/ws";
pub const PUSH_MSG_PATH: &str = "/push_msg";
pub const HEADERS_PATH: &str = "/headers";
pub const SLOW_PATH: &str = "/slow";
pub const STATUS_PATH: &str = "/status";
pub const JSON_PATH: &str = "/json";
pub const POST_ECHO_PATH: &str = "/post_echo";
pub const TIMEOUT_PATH: &str = "/timeout";

pub enum MockPath {
    Hello,
    Gzip,
    Echo,
    PushMsg,
    Websocket,
    Headers,
    Slow,
    Status,
    Json,
    PostEcho,
    Timeout,
    NotFound,
}

pub static HTTP_PATH_LIST: [MockPath; 10] = [
    MockPath::Hello,
    MockPath::Gzip,
    MockPath::Echo,
    MockPath::PushMsg,
    MockPath::Headers,
    MockPath::Slow,
    MockPath::Status,
    MockPath::Json,
    MockPath::PostEcho,
    MockPath::Timeout,
];

pub static WS_PATH: MockPath = MockPath::Websocket;

impl From<&str> for MockPath {
    fn from(value: &str) -> Self {
        match value {
            HELLO_PATH => MockPath::Hello,
            GZIP_PATH => MockPath::Gzip,
            ECHO_PATH => MockPath::Echo,
            PUSH_MSG_PATH => MockPath::PushMsg,
            WEBSOCKET_PATH => MockPath::Websocket,
            HEADERS_PATH => MockPath::Headers,
            SLOW_PATH => MockPath::Slow,
            STATUS_PATH => MockPath::Status,
            JSON_PATH => MockPath::Json,
            POST_ECHO_PATH => MockPath::PostEcho,
            TIMEOUT_PATH => MockPath::Timeout,
            _ => MockPath::NotFound,
        }
    }
}

impl Display for MockPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MockPath::Hello => write!(f, "{}", HELLO_PATH),
            MockPath::Gzip => write!(f, "{}", GZIP_PATH),
            MockPath::Echo => write!(f, "{}", ECHO_PATH),
            MockPath::PushMsg => write!(f, "{}", PUSH_MSG_PATH),
            MockPath::Websocket => write!(f, "{}", WEBSOCKET_PATH),
            MockPath::Headers => write!(f, "{}", HEADERS_PATH),
            MockPath::Slow => write!(f, "{}", SLOW_PATH),
            MockPath::Status => write!(f, "{}", STATUS_PATH),
            MockPath::Json => write!(f, "{}", JSON_PATH),
            MockPath::PostEcho => write!(f, "{}", POST_ECHO_PATH),
            MockPath::Timeout => write!(f, "{}", TIMEOUT_PATH),
            MockPath::NotFound => write!(f, "/"),
        }
    }
}

#[instrument(skip(req))]
pub async fn mock_server_fn(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
    let res = match (req.method(), MockPath::from(req.uri().path())) {
        (&Method::GET, MockPath::Websocket) => {
            let (res, ws) = hyper_tungstenite::upgrade(req, None)?;

            tokio::spawn(async move {
                let mut ws = ws.await.unwrap();

                while let Some(msg) = ws.next().await {
                    trace!("websocket msg: {:?}", msg);
                    match msg {
                        Ok(msg) => match msg {
                            Message::Binary(data) => {
                                ws.send(Message::Binary(data)).await.unwrap();
                            }
                            Message::Text(data) => {
                                ws.send(Message::Text(data)).await.unwrap();
                            }
                            Message::Ping(data) => {
                                ws.send(Message::Pong(data)).await.unwrap();
                            }
                            Message::Pong(_) => {}
                            Message::Close(_) => {}
                            Message::Frame(_) => {}
                        },
                        _ => {
                            warn!("websocket msg error: {:?}", msg);
                        }
                    }
                }
            });

            let (parts, body) = res.into_parts();
            let bytes = body.collect().await?.to_bytes();
            let body = Full::new(bytes).map_err(|err| anyhow!("{err}")).boxed();
            let res_result = Response::from_parts(parts, body);
            Ok(res_result)
        }
        (&Method::GET, MockPath::Hello) => {
            let res = Response::new(
                Full::new(Bytes::from(HELLO_WORLD))
                    .map_err(|err| anyhow!("{err}"))
                    .boxed(),
            );
            Ok(res)
        }
        (&Method::GET, MockPath::Gzip) => {
            let stream_body = StreamBody::new(
                ReaderStream::new(GzipEncoder::new(HELLO_WORLD.as_bytes()))
                    .map_ok(Frame::data)
                    .map_err(|err| anyhow!("{err}")),
            );
            let res = Response::builder()
                .header(CONTENT_ENCODING, "gzip")
                .status(StatusCode::OK)
                .body(BoxBody::new(stream_body))?;
            Ok(res)
        }
        (&_, MockPath::Echo) => {
            let content_type = req.headers().get(CONTENT_TYPE).cloned();
            let bytes = req.collect().await?.to_bytes();
            let body = Full::new(bytes).map_err(|err| anyhow!("{err}")).boxed();
            let mut res = Response::new(body);
            if let Some(content_type) = content_type {
                res.headers_mut().insert(CONTENT_TYPE, content_type);
            }

            Ok(res)
        }
        (&Method::GET, MockPath::PushMsg) => {
            let (tx, rx) = broadcast::channel(1);
            tokio::spawn(async move {
                let mut interval = interval(Duration::from_millis(200));
                let mut count = 0;
                loop {
                    count += 1;
                    if count > 10 {
                        break;
                    }
                    interval.tick().await;
                    match tx.send("push msg\n".to_string()) {
                        Ok(_) => {}
                        Err(e) => {
                            dbg!(e);
                        }
                    }
                }
            });
            let stream = BroadcastStream::new(rx);
            let stream = stream
                .map_ok(|data| Frame::data(Bytes::from(data)))
                .map_err(|err| anyhow!(err));            let body = BodyExt::boxed(StreamBody::new(stream));
            let res = Response::new(body);
            Ok(res)
        }
        (&Method::GET, MockPath::Headers) => {
            // Return request headers as JSON
            let headers_json = serde_json::json!({
                "headers": req.headers()
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("invalid")))
                    .collect::<std::collections::HashMap<_, _>>()
            });
            let body = Full::new(Bytes::from(headers_json.to_string()))
                .map_err(|err| anyhow!("{err}"))
                .boxed();
            let mut res = Response::new(body);
            res.headers_mut().insert(CONTENT_TYPE, "application/json".parse().unwrap());
            Ok(res)
        }
        (&Method::GET, MockPath::Slow) => {
            // Slow endpoint that takes 3 seconds to respond
            tokio::time::sleep(Duration::from_secs(3)).await;
            let res = Response::new(
                Full::new(Bytes::from("Slow response"))
                    .map_err(|err| anyhow!("{err}"))
                    .boxed(),
            );
            Ok(res)
        }
        (&Method::GET, MockPath::Status) => {
            // Return status based on query parameter
            let query = req.uri().query().unwrap_or("");
            let status_code = if query.contains("code=") {
                let code_str = query.split("code=").nth(1).unwrap_or("200").split('&').next().unwrap_or("200");
                code_str.parse::<u16>().unwrap_or(200)
            } else {
                200
            };
            
            let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK);
            let body = Full::new(Bytes::from(format!("Status: {}", status_code)))
                .map_err(|err| anyhow!("{err}"))
                .boxed();
            let mut res = Response::new(body);
            *res.status_mut() = status;
            Ok(res)
        }
        (&Method::GET, MockPath::Json) => {
            // Return JSON response
            let json_response = serde_json::json!({
                "message": "Hello from JSON endpoint",
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                "data": {
                    "items": [1, 2, 3, 4, 5],
                    "success": true
                }
            });
            let body = Full::new(Bytes::from(json_response.to_string()))
                .map_err(|err| anyhow!("{err}"))
                .boxed();
            let mut res = Response::new(body);
            res.headers_mut().insert(CONTENT_TYPE, "application/json".parse().unwrap());
            Ok(res)
        }
        (&Method::POST, MockPath::PostEcho) => {
            // POST echo endpoint that returns the request body
            let content_type = req.headers().get(CONTENT_TYPE).cloned();
            let bytes = req.collect().await?.to_bytes();
            let body = Full::new(bytes).map_err(|err| anyhow!("{err}")).boxed();
            let mut res = Response::new(body);
            if let Some(content_type) = content_type {
                res.headers_mut().insert(CONTENT_TYPE, content_type);
            }
            Ok(res)
        }
        (&Method::GET, MockPath::Timeout) => {
            // Endpoint that takes a very long time to respond (10 seconds)
            tokio::time::sleep(Duration::from_secs(10)).await;
            let res = Response::new(
                Full::new(Bytes::from("This took a long time"))
                    .map_err(|err| anyhow!("{err}"))
                    .boxed(),
            );
            Ok(res)
        }
        _ => {
            let mut res = Response::default();
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    };
    tracing::trace!("{:?}", res);
    res
}
