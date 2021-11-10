pub(crate) mod api;
pub(crate) mod auth;
pub(crate) mod maker;
pub(crate) mod retry;

use crate::data::{Request, RequestEnvelope, ResponseEnvelope};
use crate::error::Error;
use tower::{Service, ServiceExt};

pub use crate::service::api::{ApiService, TungsteniteApiService};
pub use crate::service::auth::{Authentication, AuthenticationLayer, ResponseWithToken};
pub use crate::service::maker::MakeApiService;
pub use crate::service::retry::RetryPolicy;

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
