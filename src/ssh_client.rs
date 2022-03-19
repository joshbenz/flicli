use futures::{Stream, TryFutureExt};
use std::future;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use thrussh::{client, ChannelId, Disconnect, Pty};
use thrussh_keys::key;
use tracing::{info, instrument};
use tracing_subscriber::EnvFilter;

pub mod error {
    #[derive(thiserror::Error, Debug)]
    pub enum ClientError {
        #[error("Ssh error")]
        Ssh {
            #[from]
            source: thrussh::Error,
        },
        #[error("Ssh error")]
        SshKey {
            #[from]
            source: thrussh_keys::Error,
        },
        #[error("Ssh error")]
        Auth {
            #[from]
            source: thrussh::AgentAuthError,
        },
        #[error("IO Error")]
        IO {
            #[from]
            source: std::io::Error,
        },
    }
}

#[derive(Debug)]
struct ClientHandler {}

impl client::Handler for ClientHandler {
    type Error = error::ClientError;
    type FutureUnit = future::Ready<Result<(Self, client::Session), Self::Error>>;
    type FutureBool = future::Ready<Result<(Self, bool), Self::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        future::ready(Ok((self, b)))
    }

    fn finished(self, session: client::Session) -> Self::FutureUnit {
        future::ready(Ok((self, session)))
    }

    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        self.finished_bool(true)
    }
}

pub struct Client {
    c: client::Channel,
}

impl Client {
    pub async fn connect(
        username: impl Into<String>,
        password: impl Into<String>,
        url: impl ToSocketAddrs,
    ) -> Result<Self, String> {
        let config = thrussh::client::Config::default();
        let config = Arc::new(config);
        let ssh_client = ClientHandler {};
        let mut session = thrussh::client::connect(config, url, ssh_client)
            .await
            .unwrap();

        let auth = session
            .authenticate_password(username.into(), password.into())
            .await
            .unwrap();
        let mut channel_direct = session.channel_open_session().await.unwrap();
        let _ = channel_direct
            .request_pty(
                false,
                "",
                0,
                0,
                0,
                0,
                &[(Pty::TTY_OP_OSPEED, 14400), (Pty::TTY_OP_ISPEED, 14400)],
            )
            .await
            .unwrap();
        let _ = channel_direct.request_shell(false).await.unwrap();

        Ok(Self { c: channel_direct })
    }

    pub async fn send_command(&mut self, cmd: Vec<u8>) -> Result<String, String> {
        let k = String::from("token\r\n[jbenz@voyager ~]$ ");
        let mut buf = Vec::new();
        let mut out = String::with_capacity(cmd.len());

        self.c.data(&*cmd).await.unwrap();
        let mut counter = 0;

        while let Some(msg) = self.c.wait().await {
            counter += 1;
            match msg {
                thrussh::ChannelMsg::Data { ref data } => {
                    //let mut s = std::io::stdout();
                    data.write_all_from(0, &mut buf).unwrap();
                    out.push_str(std::str::from_utf8(&data).unwrap());
                    if out.contains(&k) {
                        return Ok(out);
                    }
                    //info!(data);
                }
                thrussh::ChannelMsg::ExtendedData { ref data, ext } => {
                    let mut s = std::io::stdout();
                    data.write_all_from(0, &mut s).unwrap();
                }
                thrussh::ChannelMsg::Eof => {
                    //info!("EOF");
                    break;
                }
                thrussh::ChannelMsg::Close => {
                    println!("CLOSED")
                }
                msg => {
                    //info!(?msg)
                }
            }
        }
        Ok(out)
    }
}
