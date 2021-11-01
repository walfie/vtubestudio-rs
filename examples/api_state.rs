use vtubestudio::data::*;
use vtubestudio::{Client, Result};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut client = Client::connect("ws://localhost:8001").await?;

    let resp = client.send(ApiStateRequest {}).await?;
    println!("{:#?}", resp);

    let resp = client
        .send(AuthenticationRequest {
            plugin_name: "name".into(),
            plugin_developer: "dev".into(),
            authentication_token: "123".into(),
        })
        .await?;

    println!("{:#?}", resp);

    Ok(())
}
