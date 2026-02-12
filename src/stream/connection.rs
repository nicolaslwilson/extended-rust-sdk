use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;

use crate::error::{X10Error, X10Result};

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

/// A WebSocket stream connection that deserializes messages into type T.
pub struct StreamConnection<T> {
    ws: WsStream,
    msgs_count: u64,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> StreamConnection<T> {
    pub(crate) async fn connect(url: &str, api_key: Option<&str>) -> X10Result<Self> {
        let mut request = url.into_client_request().map_err(|e| {
            X10Error::Other(format!("Failed to create WebSocket request: {}", e))
        })?;

        let headers = request.headers_mut();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static(concat!(
                "ExtendedRustTradingClient/",
                env!("CARGO_PKG_VERSION")
            )),
        );
        if let Some(key) = api_key {
            headers.insert(
                "X-Api-Key",
                HeaderValue::from_str(key)
                    .map_err(|e| X10Error::Other(format!("Invalid API key header: {}", e)))?,
            );
        }

        let (ws, _response) = tokio_tungstenite::connect_async(request).await?;

        Ok(Self {
            ws,
            msgs_count: 0,
            _phantom: PhantomData,
        })
    }

    /// Send a text message over the WebSocket.
    pub async fn send(&mut self, data: &str) -> X10Result<()> {
        self.ws
            .send(Message::Text(data.to_string()))
            .await
            .map_err(X10Error::WebSocket)
    }

    /// Receive and deserialize the next message.
    pub async fn recv(&mut self) -> X10Result<T> {
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    self.msgs_count += 1;
                    let parsed: T = serde_json::from_str(&text)?;
                    return Ok(parsed);
                }
                Some(Ok(Message::Ping(data))) => {
                    let _ = self.ws.send(Message::Pong(data)).await;
                }
                Some(Ok(Message::Close(_))) => {
                    return Err(X10Error::Other("WebSocket closed".into()));
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(X10Error::WebSocket(e)),
                None => return Err(X10Error::Other("WebSocket stream ended".into())),
            }
        }
    }

    /// Close the WebSocket connection.
    pub async fn close(&mut self) -> X10Result<()> {
        self.ws.close(None).await.map_err(X10Error::WebSocket)
    }

    pub fn msgs_count(&self) -> u64 {
        self.msgs_count
    }
}

impl<T: DeserializeOwned + Unpin> Stream for StreamConnection<T> {
    type Item = X10Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.ws).poll_next(cx) {
            Poll::Ready(Some(Ok(Message::Text(text)))) => {
                self.msgs_count += 1;
                match serde_json::from_str::<T>(&text) {
                    Ok(parsed) => Poll::Ready(Some(Ok(parsed))),
                    Err(e) => Poll::Ready(Some(Err(X10Error::Json(e)))),
                }
            }
            Poll::Ready(Some(Ok(Message::Close(_)))) => Poll::Ready(None),
            Poll::Ready(Some(Ok(_))) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(X10Error::WebSocket(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
