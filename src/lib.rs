#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]

//! A library for interacting with the [VTube Studio API].
//!
//! This crate exposes a [`Client`] for making requests to the VTube Studio websocket API, and
//! handles the work of mapping requests to responses (using [`tokio_tower::multiplex`]).
//!
//! The client wraps a set of configurable [`tower::Service`] middleware for handling the
//! [authentication flow](crate::service::Authentication), [retries](crate::service::RetryPolicy),
//! and [reconnects](crate::service::MakeApiService), and uses [`tokio_tungstenite`] as the
//! underlying websocket transport by default.
//!
//! [VTube Studio API]: https://github.com/DenchiSoft/VTubeStudio
//!
//! # Basic usage
//!
//! The example below creates a [`Client`] using the provided [builder](ClientBuilder), which:
//!
//! * connects to `ws://localhost:8001` using [`tokio_tungstenite`](https://docs.rs/tokio_tungstenite)
//! * authenticates with an existing token (if present and valid)
//! * reconnects when disconnected, and retries the failed request on reconnection success
//! * requests a new auth token on receiving an auth error, and retries the initial failed
//!   request on authentication success
//!
#![cfg_attr(feature = "tokio-tungstenite", doc = "```no_run")]
#![cfg_attr(not(feature = "tokio-tungstenite"), doc = "```ignore")]
#![doc = include_str!("../examples/readme.rs")]
//! ```
//!
//! To send multiple outgoing requests at the same time without waiting for a request to come back,
//! you can clone the [`Client`] per request (by default, the client wraps a
//! [`tower::buffer::Buffer`] which adds an mpsc buffer in front of the underlying websocket
//! transport).
//!
//! For an example of constructing a [`Client`] manually without the builder, check the
//! [`no_middleware` example] in the repo.
//!
//! [`no_middleware` example]: https://github.com/walfie/vtubestudio-rs/blob/master/examples/no_middleware.rs
//!
//! # Events
//!
//! The [`ClientEventStream`] returned from the [`ClientBuilder`] will also return
//! [`Event`](crate::data::Event)s if we subscribe to them.
//!
//! The example below demonstrates subscribing to [`TestEvent`](crate::data::TestEvent)s, which
//! will be emitted every second.
//!
#![cfg_attr(feature = "tokio-tungstenite", doc = "```no_run")]
#![cfg_attr(not(feature = "tokio-tungstenite"), doc = "```ignore")]
#![doc = include_str!("../examples/events.rs")]
//! ```
//!
//! # Project structure
//!
//! * [`client`] provides a high level API dealing with typed [`Request`]/[`Response`] types, which wraps a... ⏎
//!   * [`service`], a stack of [`tower::Service`]s that deal with
//!     [`RequestEnvelope`]/[`ResponseEnvelope`] pairs, and wraps a... ⏎
//!     * [`transport`], which describes the underlying websocket connection stream, using a... ⏎
//!       * [`codec`] to determine how to encode/decode websocket messages
//!
//! While the provided [`ClientBuilder`] should be sufficient for most users, each of these layers
//! can be modified to add custom behavior if needed. E.g.,
//!
//! * using a different combination of tower middleware
//! * using a different websocket library
//! * adding custom request/response types
//!   * as an escape hatch, if new request types or fields are added to the API and you don't feel
//!     like waiting for them to be added to this library
//!
//! [`Request`]: crate::data::Request
//! [`Response`]: crate::data::Response
//! [`RequestEnvelope`]: crate::data::RequestEnvelope
//! [`ResponseEnvelope`]: crate::data::ResponseEnvelope
//!
//! # Optional features
//!
//! By default, the `tokio-tungstenite` feature is enabled, which includes helpers related to the
//! [`tokio_tungstenite`] websocket library. This can be disabled in your `Cargo.toml` with
//! `default-features = false`:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("vtubestudio = { version = \"", env!("CARGO_PKG_VERSION"), "\", default-features = false }")]
//! ```

/// Utilities for creating [`Client`]s.
pub mod client;

/// [`Service`](tower::Service) middleware used by [`Client`].
pub mod service;

/// Transport ([`Sink`]/[`Stream`]) types.
///
/// [`Sink`]: futures_sink::Sink
/// [`Stream`]: futures_util::Stream
pub mod transport;

/// Codecs for converting to/from websocket message types.
pub mod codec;

pub mod data;

/// Types related to error handling.
pub mod error;

// Macro for enabling `doc_cfg` on docs.rs
macro_rules! cfg_feature {
    (
        #![$meta:meta]
        $($item:item)+
    ) => {
        $(
            #[cfg($meta)]
            #[cfg_attr(docsrs, doc(cfg($meta)))]
            $item
        )*
    }
}

pub(crate) use cfg_feature;

pub use crate::client::{Client, ClientBuilder, ClientEvent, ClientEventStream};
pub use crate::error::{Error, ErrorKind, Result};

#[cfg(doctest)]
#[cfg_attr(feature = "tokio-tungstenite", doc = include_str!("../README.md"))]
pub struct ReadmeDoctests;
