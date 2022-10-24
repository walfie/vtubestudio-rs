// This is the example from README.md, unmodified

use vtubestudio::data::StatisticsRequest;
use vtubestudio::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // An auth token from a previous successful authentication request
    let stored_token = Some("...".to_string());

    let (mut client, mut events) = Client::builder()
        .auth_token(stored_token)
        .authentication("Plugin name", "Developer name", None)
        .build_tungstenite();

    tokio::spawn(async move {
        // TODO
        // This returns whenever the authentication middleware receives a new auth token.
        // We can handle it by saving it somewhere, etc.
        while let Some(token) = events.next().await {
            println!("Got new auth token: {:?}", token);
        }
    });

    // Use the client to send a `StatisticsRequest`, handling authentication if necessary.
    // The return type is inferred from the input type to be `StatisticsResponse`.
    let resp = client.send(&StatisticsRequest {}).await?;
    println!("VTube Studio has been running for {}ms", resp.uptime);

    Ok(())
}
