pub(crate) mod api;
pub(crate) mod maker;
pub(crate) mod retry;

use crate::transport::TungsteniteApiTransport;

pub use crate::service::api::ApiService;
pub use crate::service::maker::MakeApiService;
pub use crate::service::retry::{RetryOnDisconnectLayer, RetryOnDisconnectPolicy};

pub type TungsteniteApiService = ApiService<TungsteniteApiTransport>;
