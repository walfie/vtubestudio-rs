use crate::data::{Request, RequestEnvelope, Response, ResponseData, ResponseEnvelope};
use crate::error::{Error, MultiplexError};

use futures_core::TryStream;
use futures_sink::Sink;
use futures_util::TryFutureExt;
use std::convert::TryFrom;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tower::multiplex::{Client as MultiplexClient, MultiplexTransport, TagStore};
use tower::util::ServiceExt;
use tower::Service;

#[derive(Debug)]
pub struct IdTagger(usize);

impl TagStore<RequestEnvelope, ResponseEnvelope> for IdTagger {
    type Tag = String;

    fn assign_tag(mut self: Pin<&mut Self>, request: &mut RequestEnvelope) -> Self::Tag {
        let id = self.0.to_string();
        request.request_id = Some(id.clone());
        self.0 += 1;
        id
    }

    fn finish_tag(self: Pin<&mut Self>, response: &ResponseEnvelope) -> Self::Tag {
        response.request_id.clone()
    }
}

#[derive(Debug)]
pub struct Client<T>
where
    T: Sink<RequestEnvelope> + TryStream,
{
    client: MultiplexClient<
        MultiplexTransport<T, IdTagger>,
        MultiplexError<<T as TryStream>::Error, <T as Sink<RequestEnvelope>>::Error>,
        RequestEnvelope,
    >,
}

impl<T> Client<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + Send + 'static,
    <T as Sink<RequestEnvelope>>::Error: StdError + Send,
    <T as TryStream>::Error: StdError + Send,
{
    pub fn new(transport: T) -> Self {
        let tagger = IdTagger(0);

        let multiplex_transport = MultiplexTransport::new(transport, tagger);
        let client = MultiplexClient::new(multiplex_transport);

        Self { client }
    }

    pub async fn send<Req: Request>(
        &mut self,
        data: Req,
    ) -> Result<Req::Response, Error<<T as TryStream>::Error, <T as Sink<RequestEnvelope>>::Error>>
    {
        let msg = RequestEnvelope::new(data.into());

        let resp = self.client.ready().await?.call(msg).await?;

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

impl<T> Service<RequestEnvelope> for Client<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + Send + 'static,
    <T as Sink<RequestEnvelope>>::Error: StdError + Send,
    <T as TryStream>::Error: StdError + Send,
{
    type Response = ResponseEnvelope;
    type Error = Error<<T as TryStream>::Error, <T as Sink<RequestEnvelope>>::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.client.poll_ready(cx).map_err(Error::from)
    }

    fn call(&mut self, req: RequestEnvelope) -> Self::Future {
        Box::pin(self.client.call(req).map_err(Error::from))
    }
}

impl<T, Req> Service<Req> for Client<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + Send + 'static,
    Req: Request + Send,
    <T as Sink<RequestEnvelope>>::Error: StdError + Send,
    <T as TryStream>::Error: StdError + Send,
{
    type Response = Req::Response;

    type Error = Error<<T as TryStream>::Error, <T as Sink<RequestEnvelope>>::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.client.poll_ready(cx).map_err(Error::from)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let msg = RequestEnvelope::new(req.into());

        use futures_util::future::ready;

        let resp = self
            .client
            .call(msg)
            .map_err(Self::Error::from)
            .and_then(|resp| {
                ready(match Req::Response::try_from(resp.data) {
                    Ok(data) => Ok(data),
                    Err(ResponseData::ApiError(e)) => Err(Error::Api(e)),
                    Err(e) => Err(Error::UnexpectedResponse {
                        expected: Req::Response::MESSAGE_TYPE,
                        received: e,
                    }),
                })
            });

        Box::pin(resp)
    }
}
