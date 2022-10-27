// Example of receiving events

use vtubestudio::data::{EventSubscriptionRequest, TestEventConfig};
use vtubestudio::error::BoxError;
use vtubestudio::{Client, ClientEvent};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let plugin_name = "vtubestudio-rs example";
    let plugin_developer = "Walfie";
    let (mut client, mut events) = Client::builder()
        .auth_token(None)
        .authentication(plugin_name, plugin_developer, None)
        .build_tungstenite();

    let req = EventSubscriptionRequest::subscribe(&TestEventConfig {
        test_message_for_event: "Hello from vtubestudio-rs!".to_owned(),
    })?;

    println!("Please accept the permission pop-up in VTube Studio");

    while let Some(event) = events.next().await {
        println!("Received event: {:?}", event);

        // We receive a Disconnected event on startup
        if let ClientEvent::Disconnected = event {
            println!("Reconnecting...");

            // Try to resubscribe to test events
            while let Err(e) = client.send(&req).await {
                eprintln!("Failed to subscribe to test events: {e}");
                eprintln!("Retrying in 2s...");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }

    Ok(())
}
