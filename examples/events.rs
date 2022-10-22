// TODO

use vtubestudio::data::{EventSubscriptionRequest, TestEventConfig, StatisticsRequest};
use vtubestudio::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let mut input = String::new();
    println!("Please accept the permission pop-up in VTube Studio");

    let req = EventSubscriptionRequest::subscribe(&TestEventConfig {
        test_message_for_event: "Hello from vtubestudio-rs!".to_owned(),
    })?;
    dbg!(client.send(&req).await?);

    loop {
        println!("Press Enter to send a StatisticsRequest.");
        input.clear();
        std::io::stdin().read_line(&mut input)?;

        let resp = client.send(&StatisticsRequest {}).await?;
        dbg!(resp);
    }
}
