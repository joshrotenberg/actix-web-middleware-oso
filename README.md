# Actix Middleware for Oso Authorization

> `actix-web` middleware for the [Oso](https://www.osohq.com) authorization framework.

![ci](https://github.com/joshrotenberg/actix-web-middleware-oso/actions/workflows/ci.yml/badge.svg)

```toml
[dependencies]
actix-web-middleware-oso = "0.1"
actix-web = "0.4"
oso = "0.26.0"
# ...
```

```rust
async fn authorize(req: ServiceRequest, oso: Oso) -> Result<ServiceRequest, Error> {
    let action = req.method().to_string().to_uppercase();
    let resource = req.path();

    match oso.is_allowed("_actor", action, resource) {
        Ok(true) => Ok(req),
        _ => Err(ErrorUnauthorized("not allowed")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let mut oso = Oso::new();
        oso.load_str(r#"allow(_actor, action, resource) if action = "GET" and resource.starts_with("/ok/");"#)
            .unwrap();
        let authz = OsoAuthorization::new(oso, authorize);
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(authz)
            .default_service(web::to(|| HttpResponse::Ok()))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
```
