use vtubestudio::data::*;
use vtubestudio::transport::Tungstenite;
use vtubestudio::{Client, Error};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://localhost:8001";
    let (ws, _) = tokio_tungstenite::connect_async(url).await?;
    let mut client = Client::<Tungstenite<_>>::new(ws);

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
