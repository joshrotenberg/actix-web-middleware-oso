use actix_web::{test, web, App, HttpResponse, Responder};

use actix_web_middleware_oso::extractor::ExtractedOso;
use actix_web_middleware_oso::middleware::OsoAuthorization;

mod common;

async fn hello(oso: ExtractedOso) -> impl Responder {
    let user = common::User {
        name: "alice".to_string(),
    };

    if oso.is_allowed(user, "action", "resource").unwrap() {
        HttpResponse::Ok().body("cool cool")
    } else {
        HttpResponse::Unauthorized().body("nope, sorry")
    }
}

#[actix_web::test]
async fn test_oso_extractor_success() {
    let o = common::init_oso();
    let authz = OsoAuthorization::new(o, |req, _oso| async move { Ok(req) });

    let app = test::init_service(App::new().wrap(authz).route("/", web::get().to(hello))).await;
    let req = test::TestRequest::default().to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

async fn goodbye(oso: ExtractedOso) -> impl Responder {
    let user = common::User {
        name: "notalice".to_string(),
    };

    if oso.is_allowed(user, "action", "resource").unwrap() {
        HttpResponse::Ok().body("cool cool")
    } else {
        HttpResponse::Unauthorized().body("nope, sorry")
    }
}

#[actix_web::test]
#[should_panic]
async fn test_oso_extractor_failure() {
    let o = common::init_oso();
    let authz = OsoAuthorization::new(o, |req, _oso| async move { Ok(req) });

    let app = test::init_service(App::new().wrap(authz).route("/", web::get().to(goodbye))).await;
    let req = test::TestRequest::default().to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
#[should_panic]
async fn test_oso_extractor_no_oso() {
    let app = test::init_service(App::new().route("/", web::get().to(goodbye))).await;
    let req = test::TestRequest::default().to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}
