use vtubestudio::data::*;
use vtubestudio::{Client, ClientError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://localhost:8001";
    let mut client = Client::new_tungstenite(url).await?;

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
        Err(ClientError::Api(e)) => {
            assert!(e.is_auth_error());
            println!("Got expected error: {:#?}", e);
        }
        Err(e) => eprintln!("Got unexpected error: {:#?}", e),
    }

    Ok(())
}
