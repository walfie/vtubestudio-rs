// TODO

use vtubestudio::data::{EventSubscriptionRequest, TestEventConfig};
use vtubestudio::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // An auth token from a previous successful authentication request
    let stored_token = Some("...".to_string());

    let (mut client, mut new_tokens) = Client::builder()
        .auth_token(stored_token)
        .authentication("Plugin name", "Developer name", None)
        .build_tungstenite();

    tokio::spawn(async move {
        // This returns whenever the authentication middleware receives a new auth token.
        // We can handle it by saving it somewhere, etc.
        while let Some(token) = new_tokens.next().await {
            println!("Got new auth token: {}", token);
        }
    });

    let req = EventSubscriptionRequest::subscribe(&TestEventConfig {
        test_message_for_event: "Hello from vtubestudio-rs!".to_owned(),
    })?;

    let resp = client.send(&req).await?;

    dbg!(resp);

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    Ok(())
}
