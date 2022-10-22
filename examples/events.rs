// TODO

use vtubestudio::data::{EventSubscriptionRequest, TestEventConfig, StatisticsRequest};
use vtubestudio::Client;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter: Targets = "vtubestudio=debug,events=info".parse()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let (mut client, mut new_tokens) = Client::builder()
        .auth_token(None)
        .authentication("Plugin name", "Developer name", None)
        .build_tungstenite();

    tokio::spawn(async move {
        // This returns whenever the authentication middleware receives a new auth token.
        // We can handle it by saving it somewhere, etc.
        while let Some(token) = new_tokens.next().await {
            tracing::info!("Got new auth token: {}", token);
        }
    });

    let mut input = String::new();
    tracing::info!("Please accept the permission pop-up in VTube Studio");

    let req = EventSubscriptionRequest::subscribe(&TestEventConfig {
        test_message_for_event: "Hello from vtubestudio-rs!".to_owned(),
    })?;
    client.send(&req).await?;

    loop {
        tracing::info!("Press Enter to send a StatisticsRequest.");
        input.clear();
        std::io::stdin().read_line(&mut input)?;

        client.send(&StatisticsRequest {}).await?;
    }
}
