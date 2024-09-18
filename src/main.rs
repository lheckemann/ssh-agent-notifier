//! Proxies an SSH agent and tries (best-effort) to send notifications via libnotify when signing is requested and succeeds.
//! Written after I missed the blinking of my yubikey one time too often.

// Based on the proto-dumper example from ssh-agent-lib.

use clap::Parser;
use service_binding::Binding;
use ssh_agent_lib::{
    agent::bind,
    agent::Agent,
    agent::Session,
    async_trait,
    client::connect,
    error::AgentError,
    proto::{Request, Response},
};
use tokio::sync::oneshot;

struct NotifyOnSign {
    target: Box<dyn Session>,
    peer_info: Option<String>,
}

#[async_trait]
impl Session for NotifyOnSign {
    async fn handle(&mut self, message: Request) -> Result<Response, AgentError> {
        let (sender, receiver) = oneshot::channel();
        if let Request::SignRequest(req) = &message {
            let identities = self.target.request_identities().await?;
            let identity = identities.iter().find_map(|id| {
                if id.pubkey == req.pubkey {
                    return Some(id.comment.to_string())
                }
                None
            }).unwrap_or("<unknown identity>".to_string());
            let body = match &self.peer_info {
                Some(peer_info) => format!("Client: {peer_info}\nWants to use pubkey: {identity}"),
                None => format!("Unknown Client\nWants to use pubkey: {identity}"),
            };

            tokio::task::spawn_blocking(move || {
                let notification = libnotify::Notification::new("🥺👉👈 Signing request", Some(body.as_str()), None);
                let _ = notification.show();
                if receiver.blocking_recv().is_ok() {
                    let _ = notification.update("✅ Signed", body.as_str(), None);
                    let _ = notification.show();
                }
            });
        };
        let response = self.target.handle(message).await?;
        if let Response::SignResponse(_) = &response {
            let _ = sender.send(());
        }
        Ok(response)
    }
}

struct Forwarder {
    target: Binding,
}

#[cfg(unix)]
impl Agent<tokio::net::UnixListener> for Forwarder {
    fn new_session(&mut self, socket: &tokio::net::UnixStream) -> impl Session {
        let peer_desc = socket.peer_cred().map(|peer| {
            let mut process_desc = String::from("<unknown>");
            if let Some(pid) = peer.pid() {
                process_desc = format!("{pid} (unknown)");
                if let Ok(peer_process) = procfs::process::Process::new(pid) {
                    if let Ok(cmdline) = peer_process.cmdline() {
                        if let Some(exe) = cmdline.first() {
                            process_desc = format!("{pid} ({exe:?})");
                        }
                    }
                }
            }
            let uid = peer.uid();
            format!("Process {process_desc} of user {uid}")
        }).ok();
        self.create_new_session(peer_desc)
    }
}

impl Agent<tokio::net::TcpListener> for Forwarder {
    fn new_session(&mut self, _socket: &tokio::net::TcpStream) -> impl Session {
        self.create_new_session(None)
    }
}

#[cfg(windows)]
impl Agent<ssh_agent_lib::agent::NamedPipeListener> for Forwarder {
    fn new_session(
        &mut self,
        _socket: &tokio::net::windows::named_pipe::NamedPipeServer,
    ) -> impl Session {
        self.create_new_session(None)
    }
}

impl Forwarder {
    fn create_new_session(&mut self, peer_info: Option<String>) -> impl Session {
        NotifyOnSign {
            target: connect(self.target.clone().try_into().unwrap()).unwrap(),
            peer_info,
        }
    }
}

#[derive(Debug, Parser)]
struct Args {
    /// Target SSH agent to which we will proxy all requests.
    #[clap(long)]
    target: Binding,

    /// Source that we will bind to.
    #[clap(long, short = 'H')]
    host: Binding,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    libnotify::init("notifying-ssh-agent")?;

    let args = Args::parse();

    bind(
        args.host.try_into()?,
        Forwarder {
            target: args.target,
        },
    )
    .await?;

    Ok(())
}
