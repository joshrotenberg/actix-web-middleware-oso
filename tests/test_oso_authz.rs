use actix_web::{test, web, App, HttpResponse, Responder};
use oso::{Oso, PolarClass};

use actix_web_middleware_oso::OsoAuthorization;

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug, PolarClass)]
struct User {
    #[polar(attribute)]
    name: String,
}

#[actix_web::test]
async fn test_oso_authz_success() {
    let mut o = Oso::new();
    o.register_class(User::get_polar_class()).unwrap();
    o.load_str(r#"allow(actor, _action, _resource) if actor matches User{name: "alice"};"#)
        .unwrap();

    let authz = OsoAuthorization::new(o, |req, oso| async move {
        let user = User {
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
    let mut o = Oso::new();
    o.register_class(User::get_polar_class()).unwrap();
    o.load_str(r#"allow(actor, _action, _resource) if actor matches User{name: "alice"};"#)
        .unwrap();

    let authz = OsoAuthorization::new(o, |req, oso| async move {
        if oso
            .is_allowed(
                User {
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
