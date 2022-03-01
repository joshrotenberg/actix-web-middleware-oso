use std::error::Error;
use std::fmt;

use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;

#[derive(Debug)]
pub struct AuthorizationError {
    status_code: StatusCode,
    message: String,
}

impl AuthorizationError {
    pub fn new() -> AuthorizationError {
        AuthorizationError {
            status_code: StatusCode::UNAUTHORIZED,
            message: String::from("sup"),
        }
    }
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.status_code, f)
    }
}

impl Error for AuthorizationError {}

impl ResponseError for AuthorizationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code)
            .finish()
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }
}

#[cfg(test)]
mod tests {
    use actix_web::Error;

    use super::*;

    #[test]
    fn test_authorization_error() {
        let ae = AuthorizationError::new();
        let expected = ae.status_code;

        dbg!(&ae);
        // Converting the AuthenticationError into a ResponseError should preserve the status code.
        let e = Error::from(ae);
        dbg!(&e);
        let re = e.as_response_error();
        dbg!(&re);
        assert_eq!(expected, re.status_code());
    }
}