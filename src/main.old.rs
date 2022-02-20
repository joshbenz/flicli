use std::future;
use std::sync::Arc;
use thrussh::{
    self,
    client::{self, Config, Handle, Session},
    ChannelId, ChannelOpenFailure,
};
use thrussh_keys::key;
use tokio;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

struct SSHClient {}

impl client::Handler for SSHClient {
    type Error = anyhow::Error;
    type FutureBool = future::Ready<Result<(Self, bool), anyhow::Error>>;
    type FutureUnit = future::Ready<Result<(Self, client::Session), anyhow::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        future::ready(Ok((self, b)))
    }

    fn finished(self, session: client::Session) -> Self::FutureUnit {
        future::ready(Ok((self, session)))
    }

    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        self.finished_bool(true)
    }

    fn channel_open_failure(
        self,
        channel: ChannelId,
        reason: ChannelOpenFailure,
        description: &str,
        language: &str,
        mut session: Session,
    ) -> Self::FutureUnit {
        println!("Failed to open channel");
        future::ready(Err(anyhow::anyhow!("Failed")))
    }

    fn data(self, channel: ChannelId, data: &[u8], session: Session) -> Self::FutureUnit {
        let new_data = std::str::from_utf8(data).unwrap();
        println!("message received {:?}", &new_data);
        self.finished(session)
    }
}

#[tokio::main]
async fn main() -> () {
    // use default configuration
    let config = Config::default();
    let config = Arc::new(config);
    let mvp: SSHClient = SSHClient {};
    let mut session: Handle<SSHClient>;

    let mut agent = thrussh_keys::agent::client::AgentClient::connect_env()
        .await
        .unwrap();
    let mut identities = agent.request_identities().await.unwrap();

    match thrussh::client::connect(config, "localhost:22", mvp).await {
        Ok(sess) => {
            session = sess;
            println!("Success");
        }
        Err(err) => {
            println!("Failed {}", err);
            std::process::exit(1);
        }
    }

    if session
        .authenticate_password("nullhasher", "jB092713!")
        .await
        .is_ok()
    {
        println!("logged in successfully");
    } else {
        println!("login failed",);
    }

    /*let mut channel = session
    .channel_open_direct_tcpip("localhost", 8000, "localhost", 3333)
    .await
    .unwrap();*/
    let mut channel = session.channel_open_session().await.unwrap();
    let data = b"ls -l /";
    channel.data(&data[..]).await.unwrap();
    //channel.exec(true, "ls -l /").await.unwrap();
    //let mut writer = tokio::io::BufWriter::new(session.into().unwrap());
    while let Some(msg) = channel.wait().await {
        match msg {
            thrussh::ChannelMsg::Data { ref data } => {
                let new_data = std::str::from_utf8(data.as_ref()).unwrap();
                println!("{:?}", new_data);
            }
            thrussh::ChannelMsg::Eof => {
                break;
            }
            _ => println!("idk"),
        }
    }
}
