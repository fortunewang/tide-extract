use tide::utils::async_trait;
use tide::{Request, Status, StatusCode};

#[async_trait]
pub trait FromRequest<State>: Sized {
    async fn from_request(req: &mut Request<State>) -> tide::Result<Self>;
}

pub struct Extension<T: Clone + Send + Sync + 'static>(pub T);

#[async_trait]
impl<State: Send, T: Clone + Send + Sync + 'static> FromRequest<State> for Extension<T> {
    async fn from_request(req: &mut Request<State>) -> tide::Result<Self> {
        req.ext().cloned().map(Extension).ok_or_else(|| {
            tide::Error::from_str(
                StatusCode::InternalServerError,
                format!(
                    "Cannot find extension of type `{}`",
                    std::any::type_name::<T>()
                ),
            )
        })
    }
}

pub struct Query<T: tide::convert::DeserializeOwned>(pub T);

#[async_trait]
impl<State: Send, T: tide::convert::DeserializeOwned> FromRequest<State> for Query<T> {
    async fn from_request(req: &mut Request<State>) -> tide::Result<Self> {
        req.query().map(Query)
    }
}

pub struct Path<T, const IDX: usize>(pub T)
where
    T: std::str::FromStr,
    T::Err: Into<tide::Error>;

#[async_trait]
impl<State, T, const IDX: usize> FromRequest<State> for Path<T, IDX>
where
    State: Send,
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn from_request(req: &mut Request<State>) -> tide::Result<Self> {
        let param_name = IDX.to_string();
        req.param(&param_name)?
            .parse()
            .map(Path)
            .status(StatusCode::BadRequest)
    }
}

pub struct BodyBytes(pub Vec<u8>);

#[async_trait]
impl<State: Send> FromRequest<State> for BodyBytes {
    async fn from_request(req: &mut Request<State>) -> tide::Result<Self> {
        req.body_bytes().await.map(BodyBytes)
    }
}

pub struct BodyString(pub String);

#[async_trait]
impl<State: Send> FromRequest<State> for BodyString {
    async fn from_request(req: &mut Request<State>) -> tide::Result<Self> {
        req.body_string().await.map(BodyString)
    }
}

pub struct Json<T: tide::convert::DeserializeOwned>(pub T);

#[async_trait]
impl<State: Send, T: tide::convert::DeserializeOwned> FromRequest<State> for Json<T> {
    async fn from_request(req: &mut Request<State>) -> tide::Result<Self> {
        req.body_json().await.map(Json)
    }
}

pub struct Form<T: tide::convert::DeserializeOwned>(pub T);

#[async_trait]
impl<State: Send, T: tide::convert::DeserializeOwned> FromRequest<State> for Form<T> {
    async fn from_request(req: &mut Request<State>) -> tide::Result<Self> {
        req.body_form().await.map(Form)
    }
}
