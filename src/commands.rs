use crate::server::MessageData;

use std::process::Stdio;

use tokio::process::Command;

use tokio::time::{timeout, Duration, Instant};

use futures::stream::TryStreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use warp::filters::ws::Message as WebSocketMessage;

const STATIC_BASH_SCRIPT: &'static str = include_str!(concat!(env!("OUT_DIR"), "/script.sh"));
const NEW_LINE_TIMEOUT: u64 = 120;

type Sender = tokio::sync::mpsc::Sender<WebSocketMessage>;

#[cfg(windows)]
const BASH_PATH: &'static str = "sh";
#[cfg(unix)]
const BASH_PATH: &'static str = "bash";

async fn flush(
    sink: &Sender,
    last_update: &mut Instant,
    collector: &mut Vec<String>,
) -> Result<(), tokio::sync::mpsc::error::SendError<WebSocketMessage>> {
    match sink
        .send(
            MessageData {
                message: collector.join("\n"),
                ..Default::default()
            }
            .as_websocket_message(),
        )
        .await
    {
        Ok(_) => {
            collector.clear();
            *last_update = Instant::now();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

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
    let mut collector = Vec::new();
    let mut last_update = Instant::now();
    while let Ok(Ok(Some(x))) =
        timeout(Duration::from_secs(NEW_LINE_TIMEOUT), reader.try_next()).await
    {
        if streaming {
            collector.push(x.clone());
            if collector.len() > 20 || last_update.elapsed() >= Duration::from_millis(500) {
                if let Err(e) = flush(&sink, &mut last_update, &mut collector).await {
                    eprintln!("{}", e);
                    break;
                };
            }
        } else {
            lines.push(x.clone());
        }
    }

    if streaming {
        flush(&sink, &mut last_update, &mut collector).await.ok();
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
