use super::*;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message as ClientWsMessage},
};

pub(super) async fn connect_gateway_ws(
    addr: SocketAddr,
    token: Option<&str>,
) -> Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
> {
    let uri = format!("ws://{addr}{GATEWAY_WS_ENDPOINT}");
    let mut request = uri
        .into_client_request()
        .context("failed to construct websocket request")?;
    if let Some(token) = token {
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(format!("Bearer {token}").as_str())
                .expect("valid bearer auth header"),
        );
    }
    let (socket, _) = connect_async(request)
        .await
        .context("failed to establish websocket connection")?;
    Ok(socket)
}

pub(super) async fn recv_gateway_ws_json(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> Value {
    let message = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let Some(message) = socket.next().await else {
                panic!("websocket closed before response frame");
            };
            let message = message.expect("read websocket frame");
            match message {
                ClientWsMessage::Text(text) => {
                    return serde_json::from_str::<Value>(text.as_str())
                        .expect("websocket text frame should contain json");
                }
                ClientWsMessage::Ping(payload) => {
                    socket
                        .send(ClientWsMessage::Pong(payload))
                        .await
                        .expect("send pong");
                }
                ClientWsMessage::Pong(_) => continue,
                ClientWsMessage::Binary(_) => continue,
                ClientWsMessage::Close(_) => panic!("websocket closed before json frame"),
                ClientWsMessage::Frame(_) => continue,
            }
        }
    })
    .await
    .expect("websocket response should arrive before timeout");
    message
}
