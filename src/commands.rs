use crate::server::MessageData;

use std::process::Stdio;

use tokio::process::Command;

use tokio::time::timeout;

use futures::stream::TryStreamExt;
use std::time::Duration;
use tokio_util::codec::{FramedRead, LinesCodec};
use warp::filters::ws::Message as WebSocketMessage;

const STATIC_BASH_SCRIPT: &'static str = include_str!(concat!(env!("OUT_DIR"), "/script.sh"));

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

    // let mut out_reader = BufReader::new(stdout);
    // let mut reader = FramedRead::new(stdout.chain(stderr), LinesCodec::new());
    let mut reader = FramedRead::new(stdout, LinesCodec::new());

    // tokio::spawn(async move {
    //     // let _status = child
    //     //     .wait()
    //     //     .await
    //     //     .expect("child process encountered an error");
    //     if let Err(e) = timeout(Duration::from_secs(2), child.wait()).await{
    //         println!("timeout");
    //         child.kill().await.expect("child process encountered an error");
    //         println!("killled");
    //         println!("{}", e);
    //         return Err(e)
    //     }
    //     Ok(())
    // }).await.map(|x| x.ok()).ok();

    // tokio::spawn(async move {
    //     let _status = timeout(Duration::from_secs(2), child.wait())
    //         .await
    //         .expect("child process encountered an error")
    //         .expect("child process encountered an error");

    // });

    // timeout(Duration::from_secs(2), tokio::spawn(async move {
    //     let _status = child
    //         .wait()
    //         .await
    //         .expect("child process encountered an error");
    // })).await.map(|x| x.ok()).ok();
    tokio::spawn(async move {
        let _status = child
            .wait()
            .await
            .expect("child process encountered an error");
    });

    // let mut buffer = String::new();
    // timeout(Duration::from_secs(2), reader.read_to_string(&mut buffer)).await.ok();
    // let mut lines = Vec::new();
    // while let Ok(Some(x)) = reader.try_next().await {
    //     lines.push(x)
    // }

    let mut lines = Vec::new();
    while let Ok(Ok(Some(x))) = timeout(Duration::from_secs(2), reader.try_next()).await {
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

    // let mut buffer = String::new();
    // let mut buffer = Vec::new();
    // while let Ok(x) = out_reader.read_u8().await {
    //     buffer.push(x)
    // }

    // Ok(String::from_utf8(buffer)?)

    // let mut output = String::new();
    // reader.read_to_string(&mut output).unwrap();
    // Ok(output)
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
