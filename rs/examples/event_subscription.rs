use futures::StreamExt;
use protondrive_api::{client::ApiClient, config::SdkConfig};
use protondrive_auth::login;
use protondrive_sdk::ProtonDrive;
use secrecy::SecretString;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let username = std::env::var("PROTON_USERNAME")?;
    let password = SecretString::new(std::env::var("PROTON_PASSWORD")?.into());
    let share_id: protondrive_sdk::ShareId = std::env::var("PROTON_SHARE_ID")?.into();

    let config = SdkConfig::new("external-drive-linux@0.1.0-alpha", "ProtonDrive-Rust/0.1.0")?;
    let client = ApiClient::new(config)?;
    let session = login(&client, &username, password, None).await?;
    let drive = ProtonDrive::builder().session(session).build()?;

    let since = drive.latest_event_id(&share_id).await?;
    println!("Starting event stream from {since}");

    let mut stream = drive.subscribe_events(&share_id, since).await?;
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => println!("Event: {:?}", event.action),
            Err(e) => {
                eprintln!("Stream error: {e}");
                break;
            }
        }
    }

    Ok(())
}
