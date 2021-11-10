/**
 * Copyright (c) 2019 Tower Contributors
 *
 * Permission is hereby granted, free of charge, to any
 * person obtaining a copy of this software and associated
 * documentation files (the "Software"), to deal in the
 * Software without restriction, including without
 * limitation the rights to use, copy, modify, merge,
 * publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software
 * is furnished to do so, subject to the following
 * conditions:
 *
 * The above copyright notice and this permission notice
 * shall be included in all copies or substantial portions
 * of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
 * ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
 * TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
 * PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
 * SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 * CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
 * OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
 * IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */
// Copied from `tower` for now, until it's available in a release
// https://github.com/tower-rs/tower/pull/615
use futures_util::future::BoxFuture;
use std::fmt;
use std::task::{Context, Poll};
use tower::layer::{layer_fn, LayerFn};
use tower::{Service, ServiceExt};

/// A [`Clone`] + [`Send`] boxed [`Service`].
///
/// [`CloneBoxService`] turns a service into a trait object, allowing the
/// response future type to be dynamic, and allowing the service to be cloned.
///
/// This is similar to [`BoxService`](super::BoxService) except the resulting
/// service implements [`Clone`].
///
/// # Example
///
/// ```
/// use tower::{Service, ServiceBuilder, BoxError, util::CloneBoxService};
/// use std::time::Duration;
/// #
/// # struct Request;
/// # struct Response;
/// # impl Response {
/// #     fn new() -> Self { Self }
/// # }
///
/// // This service has a complex type that is hard to name
/// let service = ServiceBuilder::new()
///     .map_request(|req| {
///         println!("received request");
///         req
///     })
///     .map_response(|res| {
///         println!("response produced");
///         res
///     })
///     .load_shed()
///     .concurrency_limit(64)
///     .timeout(Duration::from_secs(10))
///     .service_fn(|req: Request| async {
///         Ok::<_, BoxError>(Response::new())
///     });
/// # let service = assert_service(service);
///
/// // `CloneBoxService` will erase the type so it's nameable
/// let service: CloneBoxService<Request, Response, BoxError> = CloneBoxService::new(service);
/// # let service = assert_service(service);
///
/// // And we can still clone the service
/// let cloned_service = service.clone();
/// #
/// # fn assert_service<S, R>(svc: S) -> S
/// # where S: Service<R> { svc }
/// ```
pub struct CloneBoxService<T, U, E>(
    Box<
        dyn CloneService<T, Response = U, Error = E, Future = BoxFuture<'static, Result<U, E>>>
            + Send,
    >,
);

impl<T, U, E> CloneBoxService<T, U, E> {
    /// Create a new `CloneBoxService`.
    pub fn new<S>(inner: S) -> Self
    where
        S: Service<T, Response = U, Error = E> + Clone + Send + 'static,
        S::Future: Send + 'static,
    {
        let inner = inner.map_future(|f| Box::pin(f) as _);
        CloneBoxService(Box::new(inner))
    }

    /// Returns a [`Layer`] for wrapping a [`Service`] in a [`CloneBoxService`]
    /// middleware.
    ///
    /// [`Layer`]: crate::Layer
    pub fn layer<S>() -> LayerFn<fn(S) -> Self>
    where
        S: Service<T, Response = U, Error = E> + Clone + Send + 'static,
        S::Future: Send + 'static,
    {
        layer_fn(Self::new)
    }
}

impl<T, U, E> Service<T> for CloneBoxService<T, U, E> {
    type Response = U;
    type Error = E;
    type Future = BoxFuture<'static, Result<U, E>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), E>> {
        self.0.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, request: T) -> Self::Future {
        self.0.call(request)
    }
}

impl<T, U, E> Clone for CloneBoxService<T, U, E> {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

trait CloneService<R>: Service<R> {
    fn clone_box(
        &self,
    ) -> Box<
        dyn CloneService<R, Response = Self::Response, Error = Self::Error, Future = Self::Future>
            + Send,
    >;
}

impl<R, T> CloneService<R> for T
where
    T: Service<R> + Send + Clone + 'static,
{
    fn clone_box(
        &self,
    ) -> Box<dyn CloneService<R, Response = T::Response, Error = T::Error, Future = T::Future> + Send>
    {
        Box::new(self.clone())
    }
}

impl<T, U, E> fmt::Debug for CloneBoxService<T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("CloneBoxService").finish()
    }
}
