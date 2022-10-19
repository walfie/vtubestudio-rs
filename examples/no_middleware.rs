// This example demonstrates building a `Client` manually, without any middleware provided by the
// builder. This doesn't handle automatic reconnects, authentication, etc, but allows for more
// flexible control over each layer.

use vtubestudio::data::{
    ApiStateRequest, AuthenticationRequest, AuthenticationTokenRequest, StatisticsRequest,
};
use vtubestudio::service::{ApiService, TungsteniteApiService};
use vtubestudio::transport::ApiTransport;
use vtubestudio::Client;

async fn create_client(
    address: &'static str,
) -> Result<Client<TungsteniteApiService>, tokio_tungstenite::tungstenite::Error> {
    // Underlying websocket transport (deals with WS messages)
    let (ws_transport, _) = tokio_tungstenite::connect_async(address).await?;

    // API transport (deals with streams of parsed `RequestEnvelope`s and `ResponseEnvelope`s)
    let api_transport = ApiTransport::new_tungstenite(ws_transport);

    // API service (matches `RequestEnvelope`s to `ResponseEnvelope`s by request ID)
    let (service, _events) = ApiService::new(api_transport);

    // Client (deals with typed data, disregarding the envelopes)
    let client = Client::new_from_service(service);

    Ok(client)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Manually construct a client, with no middleware.
    let mut client = create_client("ws://localhost:8001").await?;

    // Now we can use the client to send requests. Here's one that doesn't require auth.
    let api_state = client.send(&ApiStateRequest {}).await?;
    dbg!(&api_state);
    assert!(!api_state.current_session_authenticated);

    // This one should fail since we're not authenticated.
    let statistics = client.send(&StatisticsRequest {}).await;
    dbg!(&statistics);
    assert!(matches!(statistics, Err(e) if e.is_unauthenticated_error()));

    let plugin_name = "vtubestudio-rs example";
    let plugin_developer = "Walfie";

    // Get a new auth token
    println!("Requesting new auth token. Please accept the pop-up in the VTube Studio app.");
    let token = client
        .send(&AuthenticationTokenRequest {
            plugin_name: plugin_name.into(),
            plugin_developer: plugin_developer.into(),
            plugin_icon: None,
        })
        .await?
        .authentication_token;

    dbg!(&token);

    // Authenticate with the token
    let auth_response = client
        .send(&AuthenticationRequest {
            plugin_name: plugin_name.into(),
            plugin_developer: plugin_developer.into(),
            authentication_token: token,
        })
        .await?;

    dbg!(&auth_response);
    assert!(auth_response.authenticated);

    // This should now succeed!
    let statistics = client.send(&StatisticsRequest {}).await?;
    dbg!(&statistics);
    println!("VTube Studio has been running for {}ms", statistics.uptime);

    Ok(())
}
