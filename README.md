# vtubestudio-rs

[![crates.io](https://img.shields.io/crates/v/vtubestudio.svg)](https://crates.io/crates/vtubestudio)
[![docs.rs](https://docs.rs/vtubestudio/badge.svg)](https://docs.rs/vtubestudio)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

A library for interacting with the [VTube Studio API].

[VTube Studio API]: https://github.com/DenchiSoft/VTubeStudio

## Basic usage

The example below creates a `Client` using the provided builder, which:

* connects to `ws://localhost:8001` using [`tokio_tungstenite`](https://docs.rs/tokio_tungstenite)
* authenticates with an existing token (if present and valid)
* reconnects when disconnected, and retries the failed request on reconnection success
* requests a new auth token on receiving an auth error, and retries the initial failed
  request on authentication success

```rust,no_run
use vtubestudio::data::StatisticsRequest;
use vtubestudio::{Client, ClientEvent, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // An auth token from a previous successful authentication request
    let stored_token = Some("...".to_string());

    let (mut client, mut events) = Client::builder()
        .auth_token(stored_token)
        .authentication("Plugin name", "Developer name", None)
        .build_tungstenite();

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                ClientEvent::NewAuthToken(new_token) => {
                    // This returns whenever the authentication middleware receives a new auth
                    // token. We can handle it by saving it somewhere, etc.
                    println!("Got new auth token: {new_token}");
                }
                _ => {
                    // Other events, such as connections/disconnections, API events, etc
                    println!("Got event: {:?}", event);
                }
            }
        }
    });

    // Use the client to send a `StatisticsRequest`, handling authentication if necessary.
    // The return type is inferred from the input type to be `StatisticsResponse`.
    let resp = client.send(&StatisticsRequest {}).await?;
    println!("VTube Studio has been running for {}ms", resp.uptime);

    Ok(())
}
```

For more details, please check the [documentation on docs.rs](https://docs.rs/vtubestudio).

