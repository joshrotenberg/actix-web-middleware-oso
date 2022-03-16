use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::{get, middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use oso::Oso;

use actix_web_middleware_oso::{ExtractedOso, OsoAuthorization};

async fn authorize(req: ServiceRequest, oso: Oso) -> Result<ServiceRequest, Error> {
    let action = req.method().to_string().to_uppercase();
    let resource = req.path();

    log::info!("checking access to {} with {}", resource, action);

    match oso.is_allowed("_actor", action, resource) {
        Ok(true) => Ok(req),
        _ => Err(ErrorUnauthorized("not allowed")),
    }
}

#[get("/extract")]
async fn index(_oso: ExtractedOso) -> impl Responder {
    "Use oso here"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        let mut oso = Oso::new();
        oso.load_str(r#"allow(_actor, action, resource) if action = "GET" and resource.starts_with("/ok/");"#)
            .unwrap();

        let authz = OsoAuthorization::new(oso, authorize);
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(authz)
            .service(index)
            .default_service(web::to(|| HttpResponse::Ok()))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
