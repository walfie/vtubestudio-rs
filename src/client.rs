use crate::data::{Request, RequestEnvelope, Response, ResponseData, ResponseEnvelope};
use crate::error::Error;

use std::convert::TryFrom;
use tower::{Service, ServiceExt};

#[derive(Debug, Clone)]
pub struct Client<S> {
    inner: S,
}

impl<S, R, W> Client<S>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope, Error = Error<R, W>>,
{
    pub fn new(service: S) -> Self {
        Self { inner: service }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    pub async fn send<Req: Request>(&mut self, data: Req) -> Result<Req::Response, Error<R, W>> {
        let msg = RequestEnvelope::new(data.into());

        let resp = self.inner.ready().await?.call(msg).await?;

        match Req::Response::try_from(resp.data) {
            Ok(data) => Ok(data),
            Err(ResponseData::ApiError(e)) => Err(Error::Api(e)),
            Err(e) => Err(Error::UnexpectedResponse {
                expected: Req::Response::MESSAGE_TYPE,
                received: e,
            }),
        }
    }
}
