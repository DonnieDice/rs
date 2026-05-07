use protondrive_api::{client::ApiClient, config::SdkConfig};
use protondrive_auth::{login, TwoFactorProvider};
use protondrive_sdk::ProtonDrive;
use secrecy::SecretString;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let username = std::env::var("PROTON_USERNAME")?;
    let password = SecretString::new(std::env::var("PROTON_PASSWORD")?.into());

    let config = SdkConfig::new("external-drive-linux@0.1.0-alpha", "ProtonDrive-Rust/0.1.0")?;
    let client = ApiClient::new(config)?;

    let session = login(&client, &username, password, None).await?;
    println!("Logged in as {} (uid={})", username, session.uid);

    let drive = ProtonDrive::builder().session(session).build()?;

    let volumes = drive.list_volumes().await?;
    println!("Volumes ({}):", volumes.len());
    for v in &volumes {
        println!("  {:?}  share={}", v.id, v.share_id);

        let children = drive.list_children(&v.share_id, None).await?;
        println!("  Root items ({}):", children.len());
        for node in children.iter().take(10) {
            println!("    [{:?}] {} ({})", node.kind, node.name, node.id);
        }
    }

    Ok(())
}
