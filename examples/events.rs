// TODO

use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use vtubestudio::data::{EventSubscriptionRequest, StatisticsRequest, TestEventConfig};
use vtubestudio::error::BoxError;
use vtubestudio::Client;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let filter: Targets = "vtubestudio=debug,events=info".parse()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let plugin_name = "vtubestudio-rs example";
    let plugin_developer = "Walfie";
    let (mut client, mut events) = Client::builder()
        .auth_token(None)
        .authentication(plugin_name, plugin_developer, None)
        .build_tungstenite();

    tokio::spawn(async move {
        // TODO
        // This returns whenever the authentication middleware receives a new auth token.
        // We can handle it by saving it somewhere, etc.
        while let Some(_event) = events.next().await {
            // TODO
        }
    });

    let mut input = String::new();
    tracing::info!("Please accept the permission pop-up in VTube Studio");

    let req = EventSubscriptionRequest::subscribe(&TestEventConfig {
        test_message_for_event: "Hello from vtubestudio-rs!".to_owned(),
    })?;
    client.send(&req).await?;

    loop {
        // TODO: This isn't really needed
        tracing::info!("Press Enter to send a StatisticsRequest.");
        input.clear();
        std::io::stdin().read_line(&mut input)?;

        client.send(&StatisticsRequest {}).await?;
    }
}
