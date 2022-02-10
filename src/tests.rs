use crate::extract::{Path, Query};
use crate::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tide::prelude::json;
use tide::StatusCode;

async fn index() -> &'static str {
    "Hello world!"
}

async fn index_alter() -> tide::Result<&'static str> {
    Ok("Hello world!")
}

async fn error() -> impl IntoResponse {
    StatusCode::Unauthorized
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct HelloResponse {
    hello: String,
}

async fn hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    crate::response::Json(HelloResponse { hello: params.name })
}

async fn get_user(Path(user_id): Path<u32, 0>) -> impl IntoResponse {
    json!({
        "user_id": user_id,
    })
}

async fn raw(_req: tide::Request<()>) -> impl IntoResponse {
    StatusCode::Ok
}

async fn original(_req: tide::Request<()>) -> tide::Result<impl IntoResponse> {
    Ok(StatusCode::Ok)
}

#[async_std::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use crate::handler::HandlerRouting;
    use tide::http::{Method, Request as HttpRequest, Response as HttpResponse, Url};

    let mut app = tide::new();
    app.at("/").handle_get(index).handle_post(index_alter);
    app.at("/error").handle_get(error);
    app.at("/hello").handle_get(hello);
    app.at("/user/:0").handle_get(get_user);
    app.at("/raw").handle_get(raw);
    app.at("/original").handle_get(original);

    let req = HttpRequest::new(Method::Get, Url::parse("http://localhost/")?);
    let res: HttpResponse = app.respond(req).await?;
    assert_eq!(res.status(), StatusCode::Ok);

    let req = HttpRequest::new(Method::Get, Url::parse("http://localhost/error")?);
    let res: HttpResponse = app.respond(req).await?;
    assert_eq!(res.status(), StatusCode::Unauthorized);

    let req = HttpRequest::new(Method::Get, Url::parse("http://localhost/hello?name=tom")?);
    let mut res: HttpResponse = app.respond(req).await?;
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(
        res.body_json::<HelloResponse>().await?,
        HelloResponse {
            hello: "tom".to_owned()
        }
    );

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct GetUserResponse {
        user_id: u32,
    }

    let req = HttpRequest::new(Method::Get, Url::parse("http://localhost/user/1001")?);
    let mut res: HttpResponse = app.respond(req).await?;
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(
        res.body_json::<GetUserResponse>().await?,
        GetUserResponse { user_id: 1001 }
    );

    Ok(())
}
