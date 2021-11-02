use crate::data::{RequestEnvelope, ResponseEnvelope};

use pin_project_lite::pin_project;
use std::pin::Pin;
use tokio_tower::multiplex::TagStore;

pub trait RequestIdGenerator: Send {
    fn generate_id(self: Pin<&mut Self>) -> String;
}

#[derive(Debug)]
pub struct NumericIdGenerator(usize);

impl NumericIdGenerator {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn new_from_id(id: usize) -> Self {
        Self(id)
    }
}

impl RequestIdGenerator for NumericIdGenerator {
    fn generate_id(mut self: Pin<&mut Self>) -> String {
        let id = self.0.to_string();
        self.0 += 1;
        id
    }
}

pin_project! {
    pub struct IdTagger<R = NumericIdGenerator> {
        #[pin] pub(crate) id_generator: R
    }
}

impl<R> TagStore<RequestEnvelope, ResponseEnvelope> for IdTagger<R>
where
    R: RequestIdGenerator,
{
    type Tag = String;

    fn assign_tag(self: Pin<&mut Self>, request: &mut RequestEnvelope) -> Self::Tag {
        let id = self.project().id_generator.generate_id();
        request.request_id = Some(id.clone());
        id
    }

    fn finish_tag(self: Pin<&mut Self>, response: &ResponseEnvelope) -> Self::Tag {
        response.request_id.clone()
    }
}
