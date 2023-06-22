use axum::{
    http::{HeaderMap, StatusCode},
    routing::post,
    Router,
};
use std::net::SocketAddr;
use std::{future::Future, pin::Pin};

pub type MachineryHandler =
    fn(String, String) -> Pin<Box<dyn Future<Output = String> + Send + 'static>>;

pub struct Machinery {
    handler: MachineryHandler,
}

impl Machinery {
    pub async fn listen(&self, addr: &SocketAddr) -> anyhow::Result<()> {
        let handler = self.handler;

        let app = Router::new().route(
            "/x",
            post(move |headers: HeaderMap, body: String| async move {
                let header = headers.get("x-machinery-service");
                let Some(header) = header else {
                    return (StatusCode::BAD_REQUEST, "{ \"error\": \"Missing service name\" }".to_owned());
                };
                let Ok(service) = header.to_str() else {
                    return (StatusCode::BAD_REQUEST, "{ \"error\": \"Invalid service name\" }".to_owned());
                };
                let output = (handler)(service.to_owned(), body).await;

                (StatusCode::OK, output)
            }),
        );

        println!("Listening on {:?}", &addr);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

impl Machinery {
    pub fn new(handler: MachineryHandler) -> Self {
        Self { handler }
    }
}
