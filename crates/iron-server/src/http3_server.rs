use axum::http::{Response, StatusCode};
use quinn::{Endpoint, ServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::{ServerConfig as TlsServerConfig, pki_types::PrivateKeyDer};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::handlers::*;
use iron_core::models::AppState;

pub struct Http3Server {
    pub state: AppState,
    pub port: u16,
}

impl Http3Server {
    pub fn new(state: AppState, port: u16) -> Self {
        Self { state, port }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let server_config = self.create_server_config()?;
        let endpoint = Endpoint::server(server_config, format!("0.0.0.0:{}", self.port).parse()?)?;

        info!("🚀 HTTP/3 server listening on port {}", self.port);

        while let Some(incoming) = endpoint.accept().await {
            let state = self.state.clone();
            tokio::spawn(async move {
                match incoming.await {
                    Ok(connection) => {
                        if let Err(e) = Self::handle_connection(connection, state).await {
                            error!("HTTP/3 connection error: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to establish QUIC connection: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(
        connection: quinn::Connection,
        state: AppState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("New HTTP/3 connection established");

        let mut h3_conn =
            h3::server::Connection::new(h3_quinn::Connection::new(connection)).await?;

        loop {
            match h3_conn.accept().await {
                Ok(Some(request_resolver)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_request(request_resolver, state).await {
                            error!("HTTP/3 request error: {}", e);
                        }
                    });
                }
                Ok(None) => {
                    debug!("HTTP/3 connection closed");
                    break;
                }
                Err(e) => {
                    error!("HTTP/3 accept error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(
        request_resolver: h3::server::RequestResolver<h3_quinn::Connection, bytes::Bytes>,
        state: AppState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (req, mut stream) = request_resolver.resolve_request().await?;
        let path = req.uri().path();
        let method = req.method();

        debug!("HTTP/3 {} {}", method, path);

        // Send response headers first
        let response = Response::builder()
            .status(StatusCode::OK)
            .header(
                "content-type",
                match path {
                    path if path.starts_with("/api/") => "application/json",
                    _ => "text/html; charset=utf-8",
                },
            )
            .body(())?;

        stream.send_response(response).await?;

        // Send body data based on path
        let body_data = match (method.as_str(), path) {
            ("GET", "/") => {
                let html = serve_index_impl(axum::extract::State(state)).await?;
                html.into_bytes()
            }
            ("GET", "/analytics") => {
                let html = serve_analytics_impl(axum::extract::State(state)).await?;
                html.into_bytes()
            }
            ("GET", "/1rm") => {
                let html = serve_onerepmax_page_impl(axum::extract::State(state)).await?;
                html.into_bytes()
            }
            ("GET", "/sharecard") => {
                let html = serve_sharecard_page_impl(axum::extract::State(state)).await?;
                html.into_bytes()
            }
            ("GET", "/api/stats") => {
                let stats = get_stats_impl(axum::extract::State(state)).await?;
                serde_json::to_vec(&stats)?
            }
            _ => {
                warn!("HTTP/3 404 Not Found: {} {}", method, path);
                b"Not Found".to_vec()
            }
        };

        stream.send_data(body_data.into()).await?;
        stream.finish().await?;

        Ok(())
    }

    fn create_server_config(&self) -> Result<ServerConfig, Box<dyn std::error::Error>> {
        // Generate self-signed certificate for development
        let cert_key = generate_simple_self_signed(vec!["localhost".into()])?;
        let cert = cert_key.cert.der().to_owned();
        let key = PrivateKeyDer::try_from(cert_key.signing_key.serialize_der())?;

        let tls_config = TlsServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)?;

        let mut server_config = ServerConfig::with_crypto(Arc::new(
            quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)?,
        ));

        // Configure transport parameters for HTTP/3
        let mut transport = quinn::TransportConfig::default();
        transport.max_concurrent_bidi_streams(100_u32.into());
        transport.max_concurrent_uni_streams(100_u32.into());
        transport.max_idle_timeout(Some(std::time::Duration::from_secs(30).try_into()?));

        server_config.transport_config(Arc::new(transport));

        Ok(server_config)
    }
}

// Helper functions to extract implementation from handlers
async fn serve_index_impl(
    state: axum::extract::State<AppState>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let result = serve_index(state).await;
    Ok(result.into_string())
}

async fn serve_analytics_impl(
    state: axum::extract::State<AppState>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let result = serve_analytics(state).await;
    Ok(result.into_string())
}

async fn serve_onerepmax_page_impl(
    state: axum::extract::State<AppState>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let result = serve_onerepmax_page(state).await;
    Ok(result.into_string())
}

async fn serve_sharecard_page_impl(
    state: axum::extract::State<AppState>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let result = serve_sharecard_page(state).await;
    Ok(result.into_string())
}

async fn get_stats_impl(
    state: axum::extract::State<AppState>,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let result = get_stats(state).await;
    Ok(result.0)
}
