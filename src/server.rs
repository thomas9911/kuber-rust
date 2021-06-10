use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// use serde_json::json;

use crate::commands;
use std::net::SocketAddr;
use std::time::SystemTime;
use warp::filters::ws::Message as WebSocketMessage;
use warp::filters::BoxedFilter;
use warp::Filter;

const FE: &'static str = include_str!("../web/dist/index.html");

type Sender = tokio::sync::mpsc::Sender<WebSocketMessage>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageData {
    pub message: String,
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub meta: Option<String>,
    // allow streaming response, otherwise collect all output before returning
    pub streaming: bool,
}

impl Default for MessageData {
    fn default() -> MessageData {
        MessageData {
            message: String::default(),
            msg_type: MessageType::Echo,
            meta: None,
            streaming: true,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    Error,
    Echo,
    Time,
    Ls,
    Sh,
    Empty,
}

impl MessageData {
    pub fn as_websocket_message(&self) -> WebSocketMessage {
        WebSocketMessage::text(serde_json::to_string(self).expect("serialization failed"))
    }

    pub fn new_error<T: std::fmt::Display>(message: T) -> MessageData {
        MessageData {
            message: message.to_string(),
            msg_type: MessageType::Error,
            ..Default::default()
        }
    }

    pub fn empty() -> MessageData {
        MessageData {
            msg_type: MessageType::Empty,
            ..Default::default()
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

// fn map_error<T: std::fmt::Display>(
//     res: Result<T, Box<dyn std::error::Error>>,
//     msg_type: MessageType,
//     meta: Option<String>,
// ) -> Result<MessageData, warp::Error> {
//     match res {
//         Err(e) => Ok(MessageData::new_error(e)),
//         Ok(data) => Ok(MessageData {
//             message: data.to_string(),
//             msg_type,
//             meta,
//             ..Default::default()
//         }),
//     }
// }

fn map_option<T: std::fmt::Display>(
    res: Option<T>,
    msg_type: MessageType,
    meta: Option<String>,
    streaming: bool,
) -> Result<MessageData, warp::Error> {
    match res {
        None => Ok(MessageData::empty()),
        Some(data) => Ok(MessageData {
            message: data.to_string(),
            msg_type,
            meta,
            streaming,
            ..Default::default()
        }),
    }
}

async fn handle_message(
    request: Result<MessageData, warp::Error>,
    sink: Sender,
    // sink: S
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
            ..Default::default()
        }),
        Ok(MessageData {
            meta,
            msg_type: MessageType::Ls,
            streaming,
            ..
        }) => map_option(commands::ls(sink).await, MessageType::Ls, meta, streaming),
        Ok(MessageData {
            meta,
            msg_type: MessageType::Sh,
            message,
            streaming,
        }) => map_option(
            commands::bash_script(&message, sink, streaming).await,
            MessageType::Sh,
            meta,
            streaming,
        ),
        Ok(msg) => Ok(msg),
    }
}

fn encode_outgoing(
    outgoing: Result<MessageData, warp::Error>,
) -> Result<WebSocketMessage, warp::Error> {
    outgoing.map(|x| x.as_websocket_message())
}

async fn handle_ws_request(websocket: warp::filters::ws::WebSocket) {
    let (mut ws_tx, ws_rx) = websocket.split();
    let (tx, mut rx) = mpsc::channel::<WebSocketMessage>(100);

    let tx_a = tx.clone();
    let a = ws_rx
        .map(decode_incoming)
        .then(move |x| handle_message(x, tx_a.clone()))
        .map(encode_outgoing);

    let mut a = Box::pin(a);

    tokio::spawn(async move {
        while let Some(b) = a.next().await {
            if let Ok(x) = b {
                if let Err(e) = tx.send(x).await {
                    eprintln!("websocket error: {:?}", e);
                }
            }
        }
    });

    while let Some(i) = rx.recv().await {
        if let Err(e) = ws_tx.send(i).await {
            eprintln!("websocket error: {:?}", e);
        }
    }

    // while let Some(b) = a.next().await {
    //     // if let Ok(msg) = b {
    //     //     if let Err(e) = ws_tx.send(msg).await {
    //     //         eprintln!("websocket error: {:?}", e);
    //     //     }
    //     // } else {
    //     //     eprintln!("websocket error: {:?}", e);
    //     // }
    //     match b {
    //         Ok(msg) => {
    //             println!("{:?}", msg);
    //             if let Err(e) = ws_tx.send(msg).await {
    //                 eprintln!("websocket error: {:?}", e);
    //             }
    //         }
    //         Err(e) => eprintln!("websocket error: {:?}", e),
    //     }
    // }
}

fn routes() -> BoxedFilter<(impl warp::Reply,)> {
    let websocket = warp::path("api")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(handle_ws_request));

    let frontend = warp::any().map(|| warp::reply::html(FE));

    websocket.or(frontend).boxed()
}

pub async fn server(addr: impl Into<SocketAddr>) {
    let api = routes();

    warp::serve(api).run(addr).await;
}
