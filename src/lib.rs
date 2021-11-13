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
//! This example creates a [`Client`] using the provided [builder](ClientBuilder), which:
//!
//! * connects to `ws://localhost:8001` using [tokio_tungstenite](https://docs.rs/tokio_tungstenite)
//! * authenticates with an existing token (if present and valid)
//! * reconnects when disconnected, and retries the failed request on reconnection success
//! * requests a new auth token on receiving an auth error, and retries the initial failed
//!   request on authentication success
//!
//! ```no_run
//! use vtubestudio::{Client, Error};
//! use vtubestudio::data::StatisticsRequest;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // An auth token from a previous successful authentication request
//!     let stored_token = Some("...".to_string());
//!
//!     let (mut client, mut new_tokens) = Client::builder()
//!         .auth_token(stored_token)
//!         .authentication("Plugin name", "Developer name", None)
//!         .build_tungstenite();
//!
//!     tokio::spawn(async move {
//!         // This returns whenever the authentication middleware receives a new auth token.
//!         // We can handle it by saving it somewhere, etc.
//!         while let Some(token) = new_tokens.next().await {
//!             println!("Got new auth token: {}", token);
//!         }
//!     });
//!
//!     // Use the client to send a `StatisticsRequest`, handling authentication if necessary.
//!     // The return type is inferred from the input type to be `StatisticsResponse`.
//!     let resp = client.send(&StatisticsRequest {}).await?;
//!     println!("VTube Studio has been running for {}ms", resp.uptime);
//!
//!     Ok(())
//! }
//! ```
//!
//! To send multiple outgoing requests at the same time without waiting for a request to come back,
//! you can clone the [`Client`] per request (by default, the client wraps a
//! [`tower::buffer::Buffer`] which adds an mpsc buffer in front of the underlying websocket
//! transport).
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

/// Request/response types for the VTube Studio API.
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

pub use crate::client::{Client, ClientBuilder, TokenReceiver};
pub use crate::error::{Error, ErrorKind, Result};
