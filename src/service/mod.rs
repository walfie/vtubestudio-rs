pub(crate) mod api;
pub(crate) mod auth;
pub(crate) mod clone_box;
pub(crate) mod maker;
pub(crate) mod retry;

use crate::data::{Request, RequestEnvelope, ResponseEnvelope};
use crate::error::Error;
use tower::{Service, ServiceExt};

pub use crate::service::api::ApiService;
pub use crate::service::auth::{Authentication, AuthenticationLayer, ResponseWithToken};
pub use crate::service::clone_box::CloneBoxService;
pub use crate::service::maker::MakeApiService;
pub use crate::service::retry::RetryPolicy;

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]
    pub use crate::service::api::TungsteniteApiService;
}

/// A [`Clone`]able [`Service`] that is compatible with [`Client`](crate::client::Client).
pub type CloneBoxApiService = CloneBoxService<RequestEnvelope, ResponseEnvelope, Error>;

/// Trait alias for a [`Service`] that is compatible with [`Client`](crate::client::Client).
pub trait ClientService:
    Service<RequestEnvelope, Response = ResponseEnvelope> + Send + Sync
where
    Error: From<Self::Error>,
{
}

impl<S> ClientService for S
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope> + Send + Sync,
    Error: From<Self::Error>,
{
}

/// Submit a request to the underlying service and parse the response.
///
/// This is the same as [`Client::send`](crate::Client::send) but as a standalone function.
pub async fn send_request<S, Req: Request>(
    service: &mut S,
    data: &Req,
) -> Result<Req::Response, Error>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    Error: From<S::Error>,
{
    let msg = RequestEnvelope::new(data)?;

    let resp = service.ready().await?.call(msg).await?;

    resp.parse::<Req::Response>()
}
