# tide-extract

Extractors for [Tide](https://github.com/http-rs/tide).

```rust
use serde::{Deserialize, Serialize};
use tide::prelude::json;
use tide::StatusCode;
use tide_extract::extract::{Path, Query};
use tide_extract::response::IntoResponse;

// return type: IntoResponse
async fn index() -> &'static str {
    "Hello world!"
}

// return type = tide::Result<T> where T: IntoResponse
async fn index_alter() -> tide::Result<&'static str> {
    Ok("Hello world!")
}

// return type = impl IntoResponse
async fn error() -> impl IntoResponse {
    StatusCode::Unauthorized
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: String,
}

#[derive(Debug, Serialize)]
struct HelloResponse {
    hello: String,
}

// use extractors (maximum of 16)
async fn hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    tide_extract::response::Json(HelloResponse { hello: params.name })
}

// extract::Path is defined using const generics
async fn get_user(Path(user_id): Path<u32, 0>) -> impl IntoResponse {
    json!({
        "user_id": user_id,
    })
}

// use tide::Request<_> as parameter instead of extractors
async fn raw(_req: tide::Request<()>) -> impl IntoResponse {
    StatusCode::Ok
}

// almost same as tide
async fn original(_req: tide::Request<()>) -> tide::Result<impl IntoResponse> {
    Ok(StatusCode::Ok)
}

fn create_app() -> tide::Server<()> {
    use tide_extract::handler::HandlerRouting;

    let mut app = tide::new();
    app.at("/").handle_get(index).handle_post(index_alter);
    app.at("/error").handle_get(error);
    app.at("/hello").handle_get(hello);
    app.at("/user/:0").handle_get(get_user);
    app.at("/raw").handle_get(raw);
    app.at("/original").handle_get(original);
    app
}
```
