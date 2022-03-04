//! Oso Authorization middleware

use std::{future::Future, rc::Rc, sync::Arc};
use std::ops::Deref;

use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use actix_web::Result;
use futures_util::future::{self, FutureExt as _, LocalBoxFuture};
use oso::Oso;

pub struct OsoAuthorization<F> {
    pub authorize_fn: Arc<F>,
    pub oso: Arc<Oso>,
}

impl<F, O> OsoAuthorization<F>
    where
        F: Fn(ServiceRequest, Oso) -> O,
        O: Future<Output=Result<ServiceRequest, Error>>,
{
    pub fn with_fn(oso: Oso, authorize_fn: F) -> OsoAuthorization<F> {
        OsoAuthorization {
            authorize_fn: Arc::new(authorize_fn),
            oso: Arc::new(oso),
        }
    }
}

impl<S, B, F, O> Transform<S, ServiceRequest> for OsoAuthorization<F>
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
        S::Future: 'static,
        F: Fn(ServiceRequest, Oso) -> O + 'static,
        O: Future<Output=Result<ServiceRequest, Error>> + 'static,
        B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = OsoAuthorizationMiddleware<S, F>;
    type InitError = ();
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(OsoAuthorizationMiddleware {
            service: Rc::new(service),
            authorize_fn: self.authorize_fn.clone(),
            oso: self.oso.clone(), // Rc::new(Oso::new()),
        })
    }
}

pub struct OsoAuthorizationMiddleware<S, F> {
    service: Rc<S>,
    authorize_fn: Arc<F>,
    oso: Arc<Oso>,
}

impl<S, B, F, O> Service<ServiceRequest> for OsoAuthorizationMiddleware<S, F>
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
        F: Fn(ServiceRequest, Oso) -> O + 'static,
        O: Future<Output=Result<ServiceRequest, Error>> + 'static,
        S::Future: 'static,
        B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let authorize_fn = Arc::clone(&self.authorize_fn);
        let oso = Arc::clone(&self.oso);
        let service = Rc::clone(&self.service);

        async move {
            let req = authorize_fn(req, oso.deref().clone()).await?;
            service.call(req).await.map(|res| res.map_into_left_body())
        }
            .boxed_local()
    }
}

#[cfg(test)]
mod tests {
    use actix_service::{into_service, Service};
    use actix_web::{error, HttpResponse};
    use actix_web::test::TestRequest;

    use super::*;

    #[actix_rt::test]
    async fn test_oso_middleware_is_ok() {
        let oso = Oso::new();
        let middleware = OsoAuthorizationMiddleware {
            service: Rc::new(into_service(|req: ServiceRequest| async move {
                Ok::<ServiceResponse, _>(req.into_response(HttpResponse::Ok().finish()))
            })),
            authorize_fn: Arc::new(|req, _oso| async { Ok(req) }),
            oso: Arc::new(oso),
        };

        let req = TestRequest::get().to_srv_request();
        let f = middleware.call(req).await;

        let _res = futures_util::future::lazy(|cx| middleware.poll_ready(cx)).await;

        assert!(f.is_ok());
    }

    #[actix_rt::test]
    async fn test_oso_middleware_is_not_ok() {
        let oso = Oso::new();
        let middleware = OsoAuthorizationMiddleware {
            service: Rc::new(into_service(|req: ServiceRequest| async move {
                Ok::<ServiceResponse, _>(req.into_response(HttpResponse::Ok().finish()))
            })),
            authorize_fn: Arc::new(|_req, _oso| async {
                Err(error::ErrorUnauthorized("none shall pass"))
            }),
            oso: Arc::new(oso),
        };

        let req = TestRequest::get().app_data(Oso::new()).to_srv_request();
        let f = middleware.call(req).await;

        let _res = futures_util::future::lazy(|cx| middleware.poll_ready(cx)).await;

        assert!(f.is_err());
    }
}
