pub(crate) mod api;
pub(crate) mod maker;

use crate::transport::TungsteniteApiTransport;

pub use crate::service::api::ApiService;
pub use crate::service::maker::MakeApiService;

pub type TungsteniteApiService = ApiService<TungsteniteApiTransport>;
