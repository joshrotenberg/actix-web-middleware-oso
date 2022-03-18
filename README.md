# Actix Middleware for Oso Authorization

> `actix-web` middleware for the [Oso](https://www.osohq.com) authorization framework.

![ci](https://github.com/joshrotenberg/actix-web-middleware-oso/actions/workflows/ci.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/actix-web-middleware-oso?label=latest)](https://crates.io/crates/actix-web-middleware-oso)
[![Documentation](https://docs.rs/actix-web-middleware-oso/badge.svg?version=0.1.0)](https://docs.rs/actix-web-middleware-oso/0.1.0)
![Apache 2.0 or MIT licensed](https://img.shields.io/crates/l/actix-web-middleware-oso)
[![Dependency Status](https://deps.rs/crate/actix-web-middleware-oso/0.1.0/status.svg)](https://deps.rs/crate/actix-web-middleware-oso/0.1.0)

## Installation

Add `actix-web-middleware-oso` as a dependency:

```toml
[dependencies]
actix-web-middleware-oso = "0.1.0"
actix-web = "4"
oso = "0.26.0"
```

## Usage

Create a function to run your Oso authorization logic.

```rust
async fn authorize(req: ServiceRequest, oso: Oso) -> Result<ServiceRequest, Error> {
    let action = req.method().to_string().to_uppercase();
    let resource = req.path();

    match oso.is_allowed("_actor", action, resource) {
        Ok(true) => Ok(req),
        _ => Err(ErrorUnauthorized("not allowed")),
    }
}
```

Initialize Oso and the middleware, and add it to your actix `App` with `wrap`.

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let mut oso = Oso::new();
        oso.load_str(r#"allow(_actor, action, resource) if action = "GET" and resource.starts_with("/ok/");"#)
            .unwrap();
        let authz = OsoMiddleware::new(oso, authorize);
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

In addition, your initialized Oso is available to handlers via the [extractor](https://actix.rs/docs/extractors/):

```rust
#[get("/hello")]
async fn hello(oso: ExtractedOso) -> impl Responder {
    let user = User {
        name: "alice".to_string(),
    };

    if oso.is_allowed(user, "action", "resource").unwrap() {
        HttpResponse::Ok().body("cool cool")
    } else {
        HttpResponse::Unauthorized().body("nope, sorry")
    }
}
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE-2.0)
  or [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))

at your option.