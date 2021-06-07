use std::process::Stdio;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command;
// const STATIC_BASH_SCRIPT: &'static str = include_str!("script.sh");
const STATIC_BASH_SCRIPT: &'static str = include_str!(concat!(env!("OUT_DIR"), "/script.sh"));

#[cfg(windows)]
const BASH_PATH: &'static str = "sh";
#[cfg(unix)]
const BASH_PATH: &'static str = "bash";

async fn run_command(cmd: &mut Command) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = cmd.stdout(Stdio::piped()).spawn().expect("failed to spawn");

    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    let mut reader = BufReader::new(stdout);

    tokio::spawn(async move {
        let _status = child
            .wait()
            .await
            .expect("child process encountered an error");
    });

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).await?;

    Ok(buffer)
}

pub async fn ls() -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("ls");
    let mut cmd = cmd.arg("-a");
    run_command(&mut cmd).await
}

pub async fn bash_script(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut user_args = shell_words::split(input)?;
    let mut args: Vec<String> = vec!["-c", STATIC_BASH_SCRIPT, "filename"]
        .into_iter()
        .map(String::from)
        .collect();
    args.append(&mut user_args);

    let mut cmd = Command::new(BASH_PATH);
    // let mut cmd = cmd.args(&["-c", STATIC_BASH_SCRIPT, "filename"]);
    let mut cmd = cmd.args(args);

    run_command(&mut cmd).await
}
