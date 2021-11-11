use crate::SQLxSession;
use futures_util::ready;
use http::Response;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower_cookies::{Cookie, Cookies};

pin_project! {
    /// Response future for [`SessionManager`].
    #[derive(Debug)]
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) future: F,
        pub(crate) session: SQLxSession,
    }
}

impl<F, ResBody, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut res = ready!(this.future.poll(cx)?);

        let cookies = res
            .extensions_mut()
            .get::<Cookies>()
            .expect("Tower_cookies extension layer missing");

        if cookies
            .get(&this.session.store().config.cookie_name[..])
            .is_none()
        {
            let cookie = Cookie::new(
                this.session.store().config.cookie_name.clone(),
                this.session.id().0.to_string(),
            );

            cookies.add(cookie);
        }

        Poll::Ready(Ok(res))
    }
}
