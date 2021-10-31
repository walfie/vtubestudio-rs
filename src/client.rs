use crate::data::*;
use crate::error::{Error, Result};

use futures_core::stream::TryStream;
use futures_sink::Sink;
use std::convert::TryFrom;
use std::pin::Pin;
use tokio_tower::multiplex::{Client as MultiplexClient, TagStore};
use tower::util::ServiceExt;
use tower::Service;
use uuid::Uuid;

struct Tagger;
impl TagStore<RequestEnvelope, ResponseEnvelope> for Tagger {
    type Tag = String;

    fn assign_tag(self: Pin<&mut Self>, request: &mut RequestEnvelope) -> Self::Tag {
        let uuid = Uuid::new_v4().to_string();
        request.request_id = Some(uuid.clone());
        uuid
    }

    fn finish_tag(self: Pin<&mut Self>, response: &ResponseEnvelope) -> Self::Tag {
        response.request_id.clone()
    }
}

type MultiplexTransport<T> = tokio_tower::multiplex::MultiplexTransport<T, Tagger>;
type TransportError<T> = tokio_tower::Error<MultiplexTransport<T>, RequestEnvelope>;

pub struct Client<T>(MultiplexClient<MultiplexTransport<T>, TransportError<T>, RequestEnvelope>)
where
    T: Sink<RequestEnvelope> + TryStream;

impl<T> Client<T>
where
    T: Sink<RequestEnvelope> + TryStream,
{
    pub fn new<U>(underlying: U) -> Client<U>
    where
        U: Sink<RequestEnvelope, Error = Error>
            + TryStream<Ok = ResponseEnvelope, Error = Error>
            + Send
            + 'static,
    {
        let client = MultiplexClient::new(MultiplexTransport::new(underlying, Tagger));
        Client(client)
    }
}

impl<T> Client<T>
where
    T: Sink<RequestEnvelope, Error = Error>
        + TryStream<Ok = ResponseEnvelope, Error = Error>
        + Send
        + 'static,
{
    pub async fn send<Req: Request>(&mut self, data: Req) -> Result<Req::Response> {
        let id = Uuid::new_v4().to_string();

        let msg = RequestEnvelope {
            api_name: "VTubeStudioPublicAPI".into(),
            api_version: "1.0".into(),
            request_id: Some(id),
            data: data.into(),
        };

        self.0
            .ready()
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        let resp = self
            .0
            .call(msg)
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_util::sync::PollSender;

    #[tokio::test]
    async fn send() -> Result<()> {
        let (req_tx, _req_rx) = mpsc::channel::<RequestEnvelope>(5);
        let (_resp_tx, resp_rx) = mpsc::channel::<Result<ResponseEnvelope>>(5);

        PollSender::new(req_tx);
        ReceiverStream::new(resp_rx);

        todo!()
    }
}
