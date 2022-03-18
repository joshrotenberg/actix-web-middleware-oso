use actix_web::{test, web, App, HttpResponse, Responder};

use actix_web_middleware_oso::middleware::OsoMiddleware;

mod common;

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::test]
async fn test_oso_authz_success() {
    let o = common::init_oso();
    let authz = OsoMiddleware::new(o, |req, oso| async move {
        let user = common::User {
            name: "alice".to_string(),
        };
        if oso.is_allowed(user, "action", "resource").unwrap() {
            Ok(req)
        } else {
            Err(actix_web::error::ErrorUnauthorized("no sir"))
        }
    });

    let app = test::init_service(App::new().wrap(authz).route("/", web::get().to(hello))).await;
    let req = test::TestRequest::default().to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
#[should_panic]
async fn test_oso_authz_failure() {
    let o = common::init_oso();
    let authz = OsoMiddleware::new(o, |req, oso| async move {
        if oso
            .is_allowed(
                common::User {
                    name: "not alice".to_string(),
                },
                "action",
                "resource",
            )
            .unwrap()
        {
            Ok(req)
        } else {
            Err(actix_web::error::ErrorUnauthorized("no sir"))
        }
    });

    let app = test::init_service(App::new().wrap(authz).route("/", web::get().to(hello))).await;
    let req = test::TestRequest::default().to_request();

    test::call_service(&app, req).await;
}
