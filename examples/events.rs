use vtubestudio::data::{EventData, EventSubscriptionRequest, TestEventConfig};
use vtubestudio::{Client, ClientEvent, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // An auth token from a previous successful authentication request
    let stored_token = Some("...".to_string());

    let (mut client, mut events) = Client::builder()
        .auth_token(stored_token)
        .authentication("Plugin name", "Developer name", None)
        .build_tungstenite();

    println!("Please accept the permission pop-up in VTube Studio");

    // Create the event subscription request, to be sent later.
    let req = EventSubscriptionRequest::subscribe(&TestEventConfig {
        test_message_for_event: "Hello from vtubestudio-rs!".to_owned(),
    })?;

    while let Some(client_event) = events.next().await {
        match client_event {
            // We receive a `Disconnected` client event whenever we are disconnected, including on
            // startup. This can be used as a cue to refresh any event subscriptions.
            ClientEvent::Disconnected => {
                println!("Connecting...");

                // Try to subscribe to test events, retrying on failure. Note that the client
                // attempts to reconnect automatically when sending a request.
                while let Err(e) = client.send(&req).await {
                    eprintln!("Failed to subscribe to test events: {e}");
                    eprintln!("Retrying in 2s...");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }

            ClientEvent::Api(EventData::Test(event)) => {
                assert_eq!(event.your_test_message, "Hello from vtubestudio-rs!");
                println!(
                    "VTube Studio has been running for {} seconds.",
                    event.counter
                );
            }

            other => {
                println!("Received event: {:?}", other)
            }
        }
    }

    Ok(())
}
