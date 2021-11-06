use tower::reconnect::Reconnect;
use tower::ServiceBuilder;
use vtubestudio::data::*;
use vtubestudio::error2::{Error, ErrorKind};
use vtubestudio::service::TungsteniteApiService;
use vtubestudio::{Client, MakeApiService};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "ws://localhost:8001";

    let service =
        Reconnect::new::<TungsteniteApiService, &str>(MakeApiService::new_tungstenite(), url);

    let service = ServiceBuilder::new()
        .retry(RetryOnDisconnect::once())
        .map_err(Error::new_custom)
        .buffer(10)
        .service(service);

    let mut client = Client::new(service);

    let mut line = String::new();
    loop {
        println!("Press Enter to send a request");
        std::io::stdin().read_line(&mut line)?;

        let resp = client.send(ApiStateRequest {}).await;

        match resp {
            Ok(resp) => println!("Received response:\n{:#?}\n", resp),
            Err(e) => println!("Received error:\n{}\n{:#?}", e, e),
        }
    }
}

use futures_util::future;
use tower::retry::Policy;

#[derive(Clone)]
struct RetryOnDisconnect {
    attempts_left: usize,
}

impl RetryOnDisconnect {
    fn once() -> Self {
        RetryOnDisconnect { attempts_left: 1 }
    }
}

impl Policy<RequestEnvelope, ResponseEnvelope, Error> for RetryOnDisconnect {
    type Future = future::Ready<Self>;

    fn retry(
        &self,
        _req: &RequestEnvelope,
        result: Result<&ResponseEnvelope, &Error>,
    ) -> Option<Self::Future> {
        match result {
            Err(e) if self.attempts_left > 0 => {
                let is_dropped = e.has_kind(ErrorKind::ConnectionDropped);

                if is_dropped {
                    eprintln!("Connection was dropped! Attempting to reconnect...");
                    Some(future::ready(Self {
                        attempts_left: self.attempts_left - 1,
                    }))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn clone_request(&self, req: &RequestEnvelope) -> Option<RequestEnvelope> {
        Some(req.clone())
    }
}
