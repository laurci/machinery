use anyhow::bail;
use axum::{
    http::{HeaderMap, StatusCode},
    routing::post,
    Router,
};
use std::net::SocketAddr;
use std::{future::Future, pin::Pin};

use crate::context::Context;

pub type MachineryHandler =
    fn(Context, String, String) -> Pin<Box<dyn Future<Output = String> + Send + 'static>>;

pub struct MachineryStandaloneConfig {
    pub listen_addr: SocketAddr,
}

pub struct Machinery {
    handler: MachineryHandler,
    standalone_config: Option<MachineryStandaloneConfig>,
}

impl Machinery {
    pub fn new(handler: MachineryHandler) -> Self {
        Self {
            handler,
            standalone_config: None,
        }
    }

    pub fn with_standalone_config(&mut self, config: MachineryStandaloneConfig) -> &mut Self {
        self.standalone_config = Some(config);
        self
    }

    pub async fn boot(&self) -> anyhow::Result<()> {
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

                let ctx = Context {
                    headers: headers.clone()
                };

                let output = (handler)(ctx, service.to_owned(), body).await;

                (StatusCode::OK, output)
            }),
        );

        if cfg!(feature = "lambda") {
            log::info!("using lambda server");

            return Ok(());
        }

        if cfg!(feature = "standalone") {
            log::info!("using standalone server");
            let Some(standalone_config) = &self.standalone_config else {
                bail!("Machinery standalone config was not provided, but standalone feature is enabled.");
            };

            log::info!("listening on {}", standalone_config.listen_addr);

            axum::Server::bind(&standalone_config.listen_addr)
                .serve(app.into_make_service())
                .await?;

            return Ok(());
        }

        bail!("No machinery serving feature is enabled.");
    }
}
