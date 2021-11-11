#![deny(missing_docs)]
#![deny(
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_debug_implementations
)]
// TODO: More thorough crate-level docs.

//! A library for interacting with the [VTube Studio API].
//!
//! [VTube Studio API]: https://github.com/DenchiSoft/VTubeStudio

/// Utilities for creating [`Client`]s.
pub mod client;

/// A collection of [`tower::Service`] middleware used by [`Client`].
pub mod service;

/// Codecs for converting to/from websocket message types.
pub mod codec;

/// Request/response types for the VTube Studio API.
pub mod data;

/// Error handling.
pub mod error;

mod transport;

mod clone_boxed;

pub use crate::client::{Client, ClientBuilder, TokenReceiver};
pub use crate::clone_boxed::CloneBoxService;
pub use crate::codec::MessageCodec;
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::service::api::ApiService;
pub use crate::service::maker::MakeApiService;
pub use crate::transport::api::ApiTransport;
pub use crate::transport::connector::TungsteniteConnector;
pub use crate::transport::TungsteniteApiTransport;
