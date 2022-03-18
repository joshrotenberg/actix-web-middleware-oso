//! Extractor for Oso

use std::borrow::Borrow;
use std::ops::Deref;

use actix_utils::future::{ready, Ready};
use actix_web::{dev::Payload, Error, FromRequest, HttpMessage, HttpRequest};
use actix_web::error::ErrorBadRequest;
use actix_web::Result;
use oso::Oso;

/// Extractor to make Oso available to handlers.
///
/// ```no_run
/// use actix_web::{get, HttpResponse, Responder};
/// use actix_web_middleware_oso::extractor::ExtractedOso;
/// use oso::PolarClass;
///
/// // ...
/// #[derive(PolarClass)]
/// struct User {
///     name: String,
/// }
///
/// // ...
///
/// #[get("/hello")]
/// async fn hello(oso: ExtractedOso) -> impl Responder {
///     let user = User {
///         name: "alice".to_string(),
///     };
///
///    if oso.is_allowed(user, "action", "resource").unwrap() {
///        HttpResponse::Ok().body("cool cool")
///     } else {
///         HttpResponse::Unauthorized().body("nope, sorry")
///     }
/// }
/// ```
#[derive(Clone, Default)]
pub struct ExtractedOso(pub Oso);

impl Deref for ExtractedOso {
    type Target = Oso;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for ExtractedOso {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(oso) = req.extensions().get::<ExtractedOso>() {
            ready(Ok(oso.borrow().clone()))
        } else {
            ready(Err(ErrorBadRequest("No Oso could be extracted")))
        }
    }
}
