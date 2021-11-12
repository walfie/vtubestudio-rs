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

/// A collection of [`Service`](tower::Service) middleware used by [`Client`].
pub mod service;

/// A collection of transport ([`Sink`]/[`Stream`]) types.
///
/// [`Sink`]: futures_sink::Sink
/// [`Stream`]: futures_util::Stream
pub mod transport;

/// Codecs for converting to/from websocket message types.
pub mod codec;

/// Request/response types for the VTube Studio API.
pub mod data;

/// Types related to error handling.
pub mod error;

pub use crate::client::{Client, ClientBuilder, TokenReceiver};
pub use crate::error::{Error, ErrorKind, Result};
