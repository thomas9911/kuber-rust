use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
// use serde_json::json;

use std::time::SystemTime;
// use tokio::time::sleep;
use warp::filters::ws::Message as WebSocketMessage;
use warp::filters::BoxedFilter;
// use warp::http::Uri;
use crate::commands;
use std::net::SocketAddr;
use warp::Filter;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MessageData {
    message: String,
    #[serde(rename = "type")]
    msg_type: MessageType,
    meta: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    Error,
    Echo,
    Time,
    Ls,
    Sh,
}

impl MessageData {
    pub fn new_error<T: std::fmt::Display>(message: T) -> MessageData {
        MessageData {
            message: message.to_string(),
            msg_type: MessageType::Error,
            meta: None,
        }
    }
}

fn decode_incoming(
    incoming: Result<WebSocketMessage, warp::Error>,
) -> Result<MessageData, warp::Error> {
    incoming.map(|x| match serde_json::from_slice(x.as_bytes()) {
        Ok(x) => x,
        Err(e) => MessageData::new_error(e),
    })
}

fn map_error<T: std::fmt::Display>(
    res: Result<T, Box<dyn std::error::Error>>,
    msg_type: MessageType,
    meta: Option<String>,
) -> Result<MessageData, warp::Error> {
    match res {
        Err(e) => Ok(MessageData::new_error(e)),
        Ok(data) => Ok(MessageData {
            message: data.to_string(),
            msg_type,
            meta,
        }),
    }
}

async fn handle_request(
    request: Result<MessageData, warp::Error>,
) -> Result<MessageData, warp::Error> {
    match request {
        Err(e) => Err(e),
        Ok(MessageData {
            msg_type: MessageType::Time,
            meta,
            ..
        }) => Ok(MessageData {
            meta,
            msg_type: MessageType::Time,
            message: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        }),
        Ok(MessageData {
            meta,
            msg_type: MessageType::Ls,
            ..
        }) => map_error(commands::ls().await, MessageType::Ls, meta),
        Ok(MessageData {
            meta,
            msg_type: MessageType::Sh,
            message,
        }) => map_error(commands::bash_script(&message).await, MessageType::Sh, meta),
        Ok(msg) => Ok(msg),
    }
}

fn encode_outgoing(
    outgoing: Result<MessageData, warp::Error>,
) -> Result<WebSocketMessage, warp::Error> {
    outgoing
        .map(|x| WebSocketMessage::text(serde_json::to_string(&x).expect("serialization failed")))
}

// fn websocket_mapper(incoming: Result<WebSocketMessage, warp::Error>) -> Result<WebSocketMessage, warp::Error> {
//     if let Ok(incoming_message) = incoming {
//         if let Ok(text_message) = incoming_message.to_str() {
//             Ok(WebSocketMessage::text(
//                 json!({"type": "echo", "message": text_message}
//                 )
//                 .to_string(),
//             ))
//         } else {
//             Ok(WebSocketMessage::text(
//                 json!({"type": "error", "message": "echo command failed"})
//                     .to_string(),
//             ))
//         }

//         // Ok(warp::reply::json(&json!({"type": "echo", "message": incoming_message.to_str()})))
//     } else {
//         incoming
//     }
// }

fn routes() -> BoxedFilter<(impl warp::Reply,)> {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name))
        .boxed();

    let websocket = warp::path("api").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| {
            let (tx, rx) = websocket.split();

            rx.map(decode_incoming)
                .then(handle_request)
                .map(encode_outgoing)
                .forward(tx)
                .map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
        })
    });

    hello.or(websocket).boxed()
}

pub async fn server(addr: impl Into<SocketAddr>) {
    let api = routes();

    warp::serve(api).run(addr).await;
}
