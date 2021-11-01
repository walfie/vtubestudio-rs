use vtubestudio::data::*;
use vtubestudio::{Client, Error, Result};

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

    // This should fail since we're not authenticated
    let resp = client.send(AvailableModelsRequest {}).await;
    match resp {
        Ok(_) => panic!("Expected auth error"),
        Err(Error::Api(e)) => {
            assert!(e.is_auth_error());
            println!("Got expected error: {:#?}", e);
        }
        Err(e) => eprintln!("Got unexpected error: {:#?}", e),
    }

    Ok(())
}
