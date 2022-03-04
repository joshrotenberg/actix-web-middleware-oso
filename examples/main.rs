use actix_web::{App, Error, HttpServer, middleware, web};
use actix_web::dev::ServiceRequest;
use oso::{Oso, PolarClass};

use actix_web_middleware_oso::OsoAuthorization;

#[derive(Debug, PolarClass)]
struct User {
    name: String,
}

async fn authorizer(req: ServiceRequest, _oso: Oso) -> Result<ServiceRequest, Error> {
    Ok(req)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let mut oso = Oso::new();
        oso.register_class(User::get_polar_class());
        let authz = OsoAuthorization::with_fn(oso, authorizer);
        App::new()
            // .app_data(oso.clone())
            .wrap(middleware::Logger::default())
            .wrap(authz)
            .service(web::resource("/").to(|| async { "Test\r\n" }))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
