use crate::extract::FromRequest;
use crate::response::IntoResponse;
use std::future::Future;
use std::marker::PhantomData;
use tide::http;
use tide::utils::async_trait;
use tide::{Endpoint, Request};

#[async_trait]
pub trait Handler<T, State: Clone + Send + Sync + 'static>: Send + Sync + 'static {
    async fn call(&self, req: Request<State>) -> tide::Result;
}

#[async_trait]
impl<State, F, Fut, Res> Handler<Request<State>, State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(Request<State>) -> Fut,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse + 'static,
{
    async fn call(&self, req: Request<State>) -> tide::Result {
        self(req).await.into_response()
    }
}

// tide::Result does not implement Into<tide::Response>
#[async_trait]
impl<State, F, Fut, Res> Handler<tide::Result<Request<State>>, State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(Request<State>) -> Fut,
    Fut: Future<Output = tide::Result<Res>> + Send + 'static,
    Res: IntoResponse + 'static,
{
    async fn call(&self, req: Request<State>) -> tide::Result {
        self(req).await?.into_response()
    }
}

#[async_trait]
impl<State, F, Fut, Res> Handler<(), State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn() -> Fut,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse + 'static,
{
    async fn call(&self, _req: Request<State>) -> tide::Result {
        self().await.into_response()
    }
}

// tide::Result does not implement Into<tide::Response>
#[async_trait]
impl<State, F, Fut, Res> Handler<tide::Result<()>, State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn() -> Fut,
    Fut: Future<Output = tide::Result<Res>> + Send + 'static,
    Res: IntoResponse + 'static,
{
    async fn call(&self, _req: Request<State>) -> tide::Result {
        self().await?.into_response()
    }
}

macro_rules! impl_handler {
    ( $($ty:ident),* $(,)? ) => {
        #[async_trait]
        #[allow(non_snake_case)]
        impl<State, F, Fut, Res, $($ty,)*> Handler<($($ty,)*), State> for F
        where
            $( $ty: FromRequest<State> + Send,)*
            State: Clone + Send + Sync + 'static,
            F: Send + Sync + 'static + Fn($($ty,)*) -> Fut,
            Fut: Future<Output = Res> + Send + 'static,
            Res: IntoResponse + 'static,
        {
            async fn call(&self, mut req: Request<State>) -> tide::Result {
                $(
                    let $ty = $ty::from_request(&mut req).await?;
                )*

                self($($ty,)*).await.into_response()
            }
        }

        // tide::Result does not implement Into<tide::Response>
        #[async_trait]
        #[allow(non_snake_case)]
        impl<State, F, Fut, Res, $($ty,)*> Handler<tide::Result<($($ty,)*)>, State> for F
        where
            $( $ty: FromRequest<State> + Send,)*
            State: Clone + Send + Sync + 'static,
            F: Send + Sync + 'static + Fn($($ty,)*) -> Fut,
            Fut: Future<Output = tide::Result<Res>> + Send + 'static,
            Res: IntoResponse + 'static,
        {
            async fn call(&self, mut req: Request<State>) -> tide::Result {
                $(
                    let $ty = $ty::from_request(&mut req).await?;
                )*

                self($($ty,)*).await?.into_response()
            }
        }
    };
}

impl_handler!(T1);
impl_handler!(T1, T2);
impl_handler!(T1, T2, T3);
impl_handler!(T1, T2, T3, T4);
impl_handler!(T1, T2, T3, T4, T5);
impl_handler!(T1, T2, T3, T4, T5, T6);
impl_handler!(T1, T2, T3, T4, T5, T6, T7);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

pub struct HandlerEndpoint<T, State, H>
where
    T: Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    H: Handler<T, State>,
{
    _t: PhantomData<T>,
    _state: PhantomData<State>,
    handler: H,
}

impl<T, State, H> HandlerEndpoint<T, State, H>
where
    T: Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    H: Handler<T, State>,
{
    pub fn new(handler: H) -> Self {
        Self {
            _t: PhantomData,
            _state: PhantomData,
            handler,
        }
    }
}

#[async_trait]
impl<T, State, H> Endpoint<State> for HandlerEndpoint<T, State, H>
where
    T: Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    H: Handler<T, State>,
{
    async fn call(&self, req: Request<State>) -> tide::Result {
        self.handler.call(req).await
    }
}

pub trait HandlerRouting<State: Clone + Send + Sync + 'static> {
    fn handle_method<T: Send + Sync + 'static>(
        &mut self,
        method: http::Method,
        handler: impl Handler<T, State>,
    ) -> &mut Self;
    fn handle_get<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Get, handler)
    }
    fn handle_head<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Head, handler)
    }
    fn handle_put<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Put, handler)
    }
    fn handle_post<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Post, handler)
    }
    fn handle_delete<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Delete, handler)
    }
    fn handle_options<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Options, handler)
    }
    fn handle_connect<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Connect, handler)
    }
    fn handle_patch<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Patch, handler)
    }
    fn handle_trace<T: Send + Sync + 'static>(
        &mut self,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.handle_method(http::Method::Trace, handler)
    }
}

impl<'a, State: Clone + Send + Sync + 'static> HandlerRouting<State> for tide::Route<'a, State> {
    fn handle_method<T: Send + Sync + 'static>(
        &mut self,
        method: http::Method,
        handler: impl Handler<T, State>,
    ) -> &mut Self {
        self.method(method, HandlerEndpoint::new(handler))
    }
}
