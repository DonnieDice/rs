// Phase 1 example — streaming download
//
// Usage:
//   PROTON_USERNAME=... PROTON_PASSWORD=... PROTON_SHARE_ID=... PROTON_NODE_ID=... \
//     cargo run --example download_stream --features rustpgp -- ./output_file

use futures::StreamExt;
use protondrive_api::{client::ApiClient, config::SdkConfig};
use protondrive_auth::login;
use protondrive_sdk::{ProtonDrive, NodeId, ShareId};
use secrecy::SecretString;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let username  = std::env::var("PROTON_USERNAME")?;
    let password  = SecretString::new(std::env::var("PROTON_PASSWORD")?.into());
    let node_id: NodeId = std::env::var("PROTON_NODE_ID")?.into();
    let out_path  = std::env::args().nth(1).expect("pass an output path as argument");

    let config = SdkConfig::new("external-drive-linux@0.1.0-alpha", "ProtonDrive-Rust/0.1.0")?;
    let client = ApiClient::new(config)?;
    let session = login(&client, &username, password, None).await?;
    let drive   = ProtonDrive::builder().session(session).build()?;

    // download() is a Phase 1 deliverable and will return NotImplemented until then
    let mut stream = drive.download(&node_id).await?;
    let mut out    = File::create(&out_path).await?;
    let mut bytes  = 0usize;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        bytes += chunk.len();
        out.write_all(&chunk).await?;
    }

    println!("Downloaded {bytes} bytes → {out_path}");
    Ok(())
}
