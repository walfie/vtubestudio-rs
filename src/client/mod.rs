use crate::data::{Request, RequestEnvelope, Response, ResponseData, ResponseEnvelope};
use crate::error::{Error, MultiplexError};

use futures_core::TryStream;
use futures_sink::Sink;
use std::convert::TryFrom;
use std::error::Error as StdError;
use std::pin::Pin;
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
