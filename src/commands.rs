use crate::server::MessageData;

use std::process::Stdio;

use tokio::process::Command;

use tokio::time::timeout;

use futures::stream::TryStreamExt;
use std::time::Duration;
use tokio_util::codec::{FramedRead, LinesCodec};
use warp::filters::ws::Message as WebSocketMessage;

const STATIC_BASH_SCRIPT: &'static str = include_str!(concat!(env!("OUT_DIR"), "/script.sh"));
const NEW_LINE_TIMEOUT: u64 = 120;

type Sender = tokio::sync::mpsc::Sender<WebSocketMessage>;

#[cfg(windows)]
const BASH_PATH: &'static str = "sh";
#[cfg(unix)]
const BASH_PATH: &'static str = "bash";

async fn run_command(cmd: &mut Command, sink: Sender, streaming: bool) -> Option<String> {
    let cmd = cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn");

    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    let mut reader = FramedRead::new(stdout, LinesCodec::new());

    tokio::spawn(async move {
        let _status = child
            .wait()
            .await
            .expect("child process encountered an error");
    });

    let mut lines = Vec::new();
    while let Ok(Ok(Some(x))) = timeout(Duration::from_secs(NEW_LINE_TIMEOUT), reader.try_next()).await {
        lines.push(x.clone());
        if streaming {
            if let Err(_e) = sink
                .send(
                    MessageData {
                        message: x,
                        ..Default::default()
                    }
                    .as_websocket_message(),
                )
                .await
            {
                eprintln!("unable to put message on websocket")
            }
        }
    }

    if streaming {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub async fn ls(sink: Sender) -> Option<String> {
    let mut cmd = Command::new("ls");
    let mut cmd = cmd.arg("-a");
    run_command(&mut cmd, sink, false).await
}

pub async fn bash_script(input: &str, sink: Sender, streaming: bool) -> Option<String> {
    let mut user_args = shell_words::split(input).ok()?;
    let mut args: Vec<String> = vec!["-c", STATIC_BASH_SCRIPT, "filename"]
        .into_iter()
        .map(String::from)
        .collect();
    args.append(&mut user_args);

    let mut cmd = Command::new(BASH_PATH);
    let mut cmd = cmd.args(args);

    run_command(&mut cmd, sink, streaming).await
}
