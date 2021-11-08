use tower::reconnect::Reconnect;
use tower::ServiceBuilder;
use vtubestudio::data::*;
use vtubestudio::error::ServiceError;
use vtubestudio::service::{
    AuthenticationLayer, ResponseWithToken, RetryOnDisconnectPolicy, TungsteniteApiService,
};
use vtubestudio::{Client, MakeApiService};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "ws://localhost:8001";

    let service =
        Reconnect::new::<TungsteniteApiService, &str>(MakeApiService::new_tungstenite(), url);

    let auth_req = AuthenticationTokenRequest {
        plugin_name: "vtubestudio-rs example".into(),
        plugin_developer: "Walfie".into(),
        plugin_icon: None,
    };

    let service = ServiceBuilder::new()
        .retry(RetryOnDisconnectPolicy::once())
        .map_response(|resp: ResponseWithToken| resp.response)
        .layer(AuthenticationLayer::new(auth_req))
        .map_err(ServiceError::from_boxed)
        .buffer(10)
        .service(service);

    let mut client = Client::new(service);

    let mut line = String::new();
    loop {
        println!("Press Enter to send a request");
        std::io::stdin().read_line(&mut line)?;

        let resp = client.send(&StatisticsRequest {}).await;

        match resp {
            Ok(resp) => println!("Received response:\n{:#?}\n", resp),
            Err(e) => println!("Received error:\n{}\n{:#?}", e, e),
        }
    }
}
