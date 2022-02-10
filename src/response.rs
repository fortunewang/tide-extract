use tide::Body;
use tide::Response;
use tide::StatusCode;

pub trait IntoResponse {
    fn into_response(self) -> tide::Result;
}

impl<T> IntoResponse for T
where
    T: TryInto<Response>,
    T::Error: Into<tide::Error>,
{
    fn into_response(self) -> tide::Result {
        self.try_into().map_err(T::Error::into)
    }
}

pub struct Json<T: tide::prelude::Serialize>(pub T);

impl<T: tide::prelude::Serialize> TryFrom<Json<T>> for Response {
    type Error = tide::Error;

    fn try_from(data: Json<T>) -> tide::Result<Self> {
        let body = Body::from_json(&data.0)?;
        Ok(Response::builder(StatusCode::Ok).body(body).build())
    }
}
