//! Oso Authorization middleware

use std::ops::Deref;
use std::{future::Future, rc::Rc, sync::Arc};

use actix_web::Result;
use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{self, FutureExt as _, LocalBoxFuture};
use oso::Oso;

/// Middleware for Oso authorization
pub struct OsoMiddleware<F> {
    oso: Rc<Oso>,
    authorize_fn: Arc<F>,
}

impl<F, O> OsoMiddleware<F>
where
    F: Fn(ServiceRequest, Oso) -> O,
    O: Future<Output = Result<ServiceRequest, Error>>,
{
    /// Create a new `OsoAuthorization`, passing in an initialized handle to Oso and a callback.
    /// `authorize_fn` will be called with the `ServiceRequest` and `Oso`.
    ///
    /// # Example
    /// ```
    /// # use actix_web::Error;
    /// # use actix_web::dev::ServiceRequest;
    /// # use actix_web::error::ErrorUnauthorized;
    /// # use oso::Oso;
    /// # use actix_web_middleware_oso::middleware::OsoMiddleware;
    ///
    /// async fn authorize(req: ServiceRequest, oso: Oso) -> Result<ServiceRequest, Error> {
    ///    let action = req.method().to_string().to_uppercase();
    ///
    ///    match oso.is_allowed("_actor", action, "_resource") {
    ///         Ok(true) => Ok(req),
    ///         _ => Err(ErrorUnauthorized("not allowed")),
    ///     }
    /// }
    ///
    /// let mut oso = Oso::new();
    /// oso.load_str(r#"allow(_actor, action, _resource) if action = "GET");"#);
    /// let authz = OsoMiddleware::new(oso, authorize);
    /// ```
    pub fn new(oso: Oso, authorize_fn: F) -> OsoMiddleware<F> {
        OsoMiddleware {
            oso: Rc::new(oso),
            authorize_fn: Arc::new(authorize_fn),
        }
    }
}

impl<S, B, F, O> Transform<S, ServiceRequest> for OsoMiddleware<F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    F: Fn(ServiceRequest, Oso) -> O + 'static,
    O: Future<Output = Result<ServiceRequest, Error>> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = OsoMiddlewareInner<S, F>;
    type InitError = ();
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(OsoMiddlewareInner {
            service: Rc::new(service),
            oso: self.oso.clone(),
            authorize_fn: self.authorize_fn.clone(),
        })
    }
}

#[doc(hidden)]
pub struct OsoMiddlewareInner<S, F> {
    service: Rc<S>,
    oso: Rc<Oso>,
    authorize_fn: Arc<F>,
}

impl<S, B, F, O> Service<ServiceRequest> for OsoMiddlewareInner<S, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    F: Fn(ServiceRequest, Oso) -> O + 'static,
    O: Future<Output = Result<ServiceRequest, Error>> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let authorize_fn = Arc::clone(&self.authorize_fn);
        let oso = Rc::clone(&self.oso);
        let service = Rc::clone(&self.service);

        async move {
            let eo = crate::extractor::ExtractedOso(oso.deref().clone());
            req.extensions_mut().insert(eo);
            let req = authorize_fn(req, oso.deref().clone()).await?;
            service.call(req).await.map(|res| res.map_into_left_body())
        }
        .boxed_local()
    }
}

#[cfg(test)]
mod tests {
    use actix_service::{into_service, Service};
    use actix_web::test::TestRequest;
    use actix_web::{error, HttpResponse};

    use super::*;

    #[actix_rt::test]
    async fn test_oso_middleware_is_ok() {
        let oso = Oso::new();
        let middleware = OsoMiddlewareInner {
            service: Rc::new(into_service(|req: ServiceRequest| async move {
                Ok::<ServiceResponse, _>(req.into_response(HttpResponse::Ok().finish()))
            })),
            authorize_fn: Arc::new(|req, _oso| async { Ok(req) }),
            oso: Rc::new(oso),
        };

        let req = TestRequest::get().to_srv_request();
        let f = middleware.call(req).await;

        let _res = futures_util::future::lazy(|cx| middleware.poll_ready(cx)).await;

        assert!(f.is_ok());
    }

    #[actix_rt::test]
    async fn test_oso_middleware_is_not_ok() {
        let oso = Oso::new();
        let middleware = OsoMiddlewareInner {
            service: Rc::new(into_service(|req: ServiceRequest| async move {
                Ok::<ServiceResponse, _>(req.into_response(HttpResponse::Ok().finish()))
            })),
            authorize_fn: Arc::new(|_req, _oso| async {
                Err(error::ErrorUnauthorized("none shall pass"))
            }),
            oso: Rc::new(oso),
        };

        let req = TestRequest::get().app_data(Oso::new()).to_srv_request();
        let f = middleware.call(req).await;

        let _res = futures_util::future::lazy(|cx| middleware.poll_ready(cx)).await;

        assert!(f.is_err());
    }
}
