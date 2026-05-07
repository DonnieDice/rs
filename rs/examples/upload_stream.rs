// Phase 2 example — streaming upload
//
// Usage:
//   PROTON_USERNAME=... PROTON_PASSWORD=... PROTON_SHARE_ID=... PROTON_PARENT_ID=... \
//     cargo run --example upload_stream --features rustpgp -- ./local_file.txt

use protondrive_api::{client::ApiClient, config::SdkConfig};
use protondrive_auth::login;
use protondrive_sdk::{ProtonDrive, NodeId, ShareId};
use protondrive_sdk::upload::UploadOptions;
use secrecy::SecretString;
use tokio::fs::File;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let username   = std::env::var("PROTON_USERNAME")?;
    let password   = SecretString::new(std::env::var("PROTON_PASSWORD")?.into());
    let parent_id: NodeId = std::env::var("PROTON_PARENT_ID")?.into();
    let local_path = std::env::args().nth(1).expect("pass a file path as argument");

    let config = SdkConfig::new("external-drive-linux@0.1.0-alpha", "ProtonDrive-Rust/0.1.0")?;
    let client = ApiClient::new(config)?;
    let session = login(&client, &username, password, None).await?;
    let drive   = ProtonDrive::builder().session(session).build()?;

    let file      = File::open(&local_path).await?;
    let file_name = std::path::Path::new(&local_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    // upload() is a Phase 2 deliverable and will return NotImplemented until then
    let node_id = drive
        .upload(&parent_id, &file_name, file, UploadOptions::default())
        .await?;

    println!("Uploaded '{file_name}' → node {node_id}");
    Ok(())
}
