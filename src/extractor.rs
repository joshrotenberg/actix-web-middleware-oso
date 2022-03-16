use std::borrow::Borrow;
use std::ops::Deref;

use actix_utils::future::{ready, Ready};
use actix_web::error::ErrorBadRequest;
use actix_web::Result;
use actix_web::{dev::Payload, Error, FromRequest, HttpMessage, HttpRequest};
use oso::Oso;

#[derive(Clone)]
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
            ready(Err(ErrorBadRequest("no oso")))
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_service::{into_service, Service};
    use actix_web::test::TestRequest;
    use actix_web::{error, HttpResponse};

    use super::*;
}
