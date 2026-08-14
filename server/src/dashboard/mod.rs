pub(crate) mod auth;
pub(crate) mod auth_routes;
pub(crate) mod model;
mod routes;

use crate::service::Service;
use anyhow::Result;
use axum::middleware;
use orbien_core::config::WebServerConfig;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn run(svc: Arc<Service>, cfg: WebServerConfig) -> Result<()> {
    let addr = format!("{}:{}", cfg.addr, cfg.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(%addr, user = %cfg.user, "webServer dashboard listening");

    // Build AuthState when rp_id / rp_origin are configured, otherwise
    // fall back to legacy Basic-Auth-only mode.
    let auth_state: Option<Arc<auth::AuthState>> =
        if !cfg.webauthn_rp_id.is_empty() && !cfg.webauthn_origin.is_empty() {
            match auth::AuthState::new(&cfg.webauthn_rp_id, &cfg.webauthn_origin) {
                Ok(a) => {
                    tracing::info!(
                        rp_id = %cfg.webauthn_rp_id,
                        "WebAuthn enabled"
                    );
                    Some(Arc::new(a))
                }
                Err(e) => {
                    tracing::warn!("WebAuthn init failed, falling back to Basic Auth: {e}");
                    None
                }
            }
        } else {
            tracing::info!("WebAuthn not configured, using Basic Auth only");
            None
        };

    let state = Arc::new(DashState {
        svc,
        cfg,
        auth: auth_state,
    });

    let app = routes::router(state.clone())
        .layer(middleware::from_fn_with_state(
            state,
            auth::auth_middleware,
        ))
        .into_make_service();

    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Clone)]
pub struct DashState {
    pub svc: Arc<Service>,
    pub cfg: WebServerConfig,
    /// Present when WebAuthn is configured; `None` → Basic Auth only.
    pub auth: Option<Arc<auth::AuthState>>,
}
