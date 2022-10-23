// TODO

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
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

    let addr = websocket_proxy().await?;

    let plugin_name = "vtubestudio-rs example";
    let plugin_developer = "Walfie";
    let (mut client, mut new_tokens) = Client::builder()
        .auth_token(None)
        .authentication(plugin_name, plugin_developer, None)
        .url(format!("ws://{}:{}", addr.ip(), addr.port()))
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

async fn websocket_proxy() -> Result<SocketAddr, BoxError> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tracing::info!(address = %addr, "Opened proxy");

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(accept_connection(stream));
        }
    });

    Ok(addr)
}

async fn accept_connection(stream: TcpStream) -> Result<(), BoxError> {
    use futures_util::{StreamExt, TryStreamExt};

    let ws_stream = tokio_tungstenite::accept_async(stream).await?;

    let (proxy_sink, proxy_stream) = ws_stream.split();
    let (vts_connection, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:8001").await?;
    tracing::info!("Connecting to VTube Studio");
    let (vts_sink, vts_stream) = vts_connection.split();

    tokio::spawn(
        proxy_stream
            .inspect_ok(|msg| tracing::info!(%msg, "Sending message"))
            .inspect_err(|error| tracing::error!(%error, "Error while sending message"))
            .forward(vts_sink),
    );
    tokio::spawn(
        vts_stream
            .inspect_ok(|msg| tracing::info!(%msg, "Receiving message"))
            .inspect_err(|error| tracing::error!(%error, "Error while receiving message"))
            .forward(proxy_sink),
    );

    Ok(())
}
