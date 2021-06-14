use serde::Serialize;
use std::net::SocketAddr;
use structopt::StructOpt;
use url::Url;

const NAME: &'static str = env!("CARGO_PKG_NAME");
// const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const BASH_DEPENDECIES: [&'static str; 4] = ["bash", "sh", "kubectl", "kubectx"];

mod commands;
mod server;

fn error_popup(text: &str) -> bool {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(NAME)
        .set_description(text)
        .show()
}

fn check_runtime_deps() -> bool {
    let not_found: Vec<_> = BASH_DEPENDECIES
        .iter()
        .filter(|x| which::which(x).is_err())
        .copied()
        .collect();
    if !not_found.is_empty() {
        error_popup(&format!("missing dependencies:\n{}", not_found.join(", ")));
        return true;
    }
    false
}

#[derive(Debug, StructOpt, Serialize)]
struct Args {
    #[structopt(long)]
    /// do not check runtime dependencies on startup
    skip_deps_check: bool,
    #[structopt(long)]
    /// do not startup a new window for the frontend
    skip_open: bool,
    #[structopt(long)]
    /// force an error popup, handy for testing if the error dialog library works
    force_error_popup: bool,
    #[structopt(long)]
    /// print the config as json on startup
    print_config: bool,
    #[structopt(long)]
    /// server address, defaults to 127.0.0.1:9894
    server: Option<SocketAddr>,
    #[structopt(long)]
    /// frontend address, defaults to the same address as the server
    frontend: Option<Url>,
    #[structopt(long)]
    /// start server in dev mode, this splits the frontend address from the server address and set some options
    dev: bool,
}

impl Args {
    pub fn resolve(mut self) -> Self {
        if self.dev {
            self.skip_deps_check = true;
            self.skip_open = true;
            self.print_config = true;
        }
        self.resolve_server();
        self
    }

    fn resolve_server(&mut self) {
        match (self.server, &self.frontend) {
            (None, None) => {
                if self.dev {
                    self.resolve_dev_server("server");
                    self.resolve_dev_server("frontend");
                    return;
                }
                self.server = Some(SocketAddr::from(([127, 0, 0, 1], 9894)));
                self.frontend = Some(Url::parse("http://localhost:9894").unwrap());
            }
            (Some(ip), None) => {
                if self.dev {
                    self.resolve_dev_server("frontend");
                    return;
                }
                self.frontend = Some(format!("http://{}", ip).parse().unwrap())
            }
            (None, Some(_)) => {
                if self.dev {
                    self.resolve_dev_server("server");
                    return;
                }
                self.server = Some(SocketAddr::from(([127, 0, 0, 1], 9894)));
            }
            (Some(_), Some(_)) => {}
        }
    }

    fn resolve_dev_server(&mut self, option: &'static str) {
        match option {
            "server" => {
                self.server = Some(SocketAddr::from(([127, 0, 0, 1], 9894)));
            }
            "frontend" => {
                self.frontend = Some(Url::parse("http://localhost:1234").unwrap());
            }
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Args::from_args().resolve();

    if config.print_config {
        println!(
            "{}",
            serde_json::to_string_pretty(&config).expect("config cannot be printed")
        );
    }

    if config.force_error_popup {
        error_popup("This is an error popup");
        return;
    }

    if !config.skip_deps_check && check_runtime_deps() {
        return;
    }

    // if open::that(" ewrfowekn flke flwefk lkwef ").is_err() {
    if !config.skip_open
        && open::that(config.frontend.expect("resolve not called").to_string()).is_err()
    {
        error_popup("Unable to open the browser");
        return;
    };

    server::server(config.server.expect("resolve not called")).await
}
