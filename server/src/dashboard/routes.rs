use super::auth_routes;
use super::model::{
    ApiResponse, ClientInfo, Page, ProxyInfo, ProxyTrafficPoint, ProxyTrafficResp, SystemConfig,
    SystemInfo, SystemStatus,
};
use super::DashState;
use crate::metrics::{ProxyTrafficHistory, TrafficWindow};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use orbien_core::VERSION;
use rust_embed::Embed;
use serde::Deserialize;
use std::path::{Component, Path as FsPath, PathBuf};
use std::sync::Arc;

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

pub fn router(state: Arc<DashState>) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/", get(index_html))
        .route("/favicon.ico", get(favicon))
        .route("/static", get(|| async { Redirect::permanent("/") }))
        .route("/static/", get(|| async { Redirect::permanent("/") }))
        .route("/static/{*path}", get(redirect_legacy_static))
        // ── auth endpoints ────────────────────────────────────────────────────
        .route("/api/v1/auth/login", post(auth_routes::login))
        .route("/api/v1/auth/logout", post(auth_routes::logout))
        .route(
            "/api/v1/auth/webauthn/register/begin",
            post(auth_routes::webauthn_register_begin),
        )
        .route(
            "/api/v1/auth/webauthn/register/finish",
            post(auth_routes::webauthn_register_finish),
        )
        .route(
            "/api/v1/auth/webauthn/login/begin",
            post(auth_routes::webauthn_login_begin),
        )
        .route(
            "/api/v1/auth/webauthn/login/finish",
            post(auth_routes::webauthn_login_finish),
        )
        // ── dashboard API ─────────────────────────────────────────────────────
        .route("/api/v1/system/info", get(system_info))
        .route("/api/v1/system/traffic", get(system_traffic))
        .route("/api/v1/clients", get(list_clients))
        .route("/api/v1/clients/{run_id}", get(get_client))
        .route("/api/v1/clients/{run_id}/kick", post(kick_client))
        .route("/api/v1/proxies", get(list_proxies))
        .route("/api/v1/proxies/{name}/traffic", get(proxy_traffic))
        .route("/{*path}", get(static_file))
        .with_state(state)
}

async fn redirect_legacy_static(Path(path): Path<String>) -> Redirect {
    let rel = path.trim_start_matches('/');
    if rel.is_empty() {
        Redirect::permanent("/")
    } else {
        Redirect::permanent(&format!("/{rel}"))
    }
}

/// Legacy Basic-Auth middleware kept for compatibility with older clients that
/// don't use the new session-cookie flow.  The new `auth_middleware` in
/// `auth.rs` supersedes this for browser-based access.
pub async fn basic_auth(
    State(state): State<Arc<DashState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    if !needs_auth(&state) {
        return Ok(next.run(req).await);
    }
    if authorized(&state, req.headers()) {
        return Ok(next.run(req).await);
    }
    let mut res = (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    res.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Basic realm=\"Restricted\""),
    );
    Err(res)
}

fn needs_auth(state: &DashState) -> bool {
    !state.cfg.user.is_empty() || !state.cfg.password.is_empty()
}

fn authorized(state: &DashState, headers: &HeaderMap) -> bool {
    let Some(h) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Some(b64) = h
        .strip_prefix("Basic ")
        .or_else(|| h.strip_prefix("basic "))
    else {
        return false;
    };
    let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) else {
        return false;
    };
    let Ok(s) = String::from_utf8(raw) else {
        return false;
    };
    let Some((u, p)) = s.split_once(':') else {
        return false;
    };
    u == state.cfg.user && p == state.cfg.password
}

async fn index_html(State(state): State<Arc<DashState>>) -> Response {
    if let Some(bytes) = load_override(&state.cfg.assets_dir, "index.html") {
        return bytes_response("text/html; charset=utf-8", bytes);
    }
    serve_asset("index.html")
}

async fn favicon(State(state): State<Arc<DashState>>) -> Response {
    if let Some(bytes) = load_override(&state.cfg.assets_dir, "favicon.ico") {
        return bytes_response("image/x-icon", bytes);
    }
    if let Some(res) = try_embedded("favicon.ico") {
        return res;
    }
    StatusCode::NOT_FOUND.into_response()
}

async fn static_file(State(state): State<Arc<DashState>>, Path(path): Path<String>) -> Response {
    let rel = path.trim_start_matches('/');
    if let Some(bytes) = load_override(&state.cfg.assets_dir, rel) {
        return bytes_response(content_type(rel), bytes);
    }
    if let Some(res) = try_embedded(rel) {
        return res;
    }
    serve_asset("index.html")
}

fn serve_asset(path: &str) -> Response {
    if let Some(res) = try_embedded(path) {
        return res;
    }
    (
        StatusCode::NOT_FOUND,
        "dashboard assets missing — run `make web` then rebuild orbien-server",
    )
        .into_response()
}

fn try_embedded(path: &str) -> Option<Response> {
    let file = Assets::get(path)?;
    Some(bytes_response(content_type(path), file.data.into_owned()))
}

#[derive(Deserialize)]
struct PageQuery {
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: usize,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 50 }

#[derive(Deserialize)]
struct TrafficQuery {
    #[serde(default)]
    range: String,
}

fn traffic_window(q: &TrafficQuery) -> TrafficWindow {
    TrafficWindow::parse(&q.range)
}

fn traffic_resp(hist: ProxyTrafficHistory) -> ProxyTrafficResp {
    ProxyTrafficResp {
        name: hist.name,
        unit: hist.unit,
        granularity: hist.granularity,
        history: hist
            .history
            .into_iter()
            .map(|p| ProxyTrafficPoint { date: p.date, traffic_in: p.traffic_in, traffic_out: p.traffic_out })
            .collect(),
    }
}

async fn system_info(State(state): State<Arc<DashState>>) -> Json<ApiResponse<SystemInfo>> {
    let snap = state.svc.dashboard_snapshot().await;
    Json(ApiResponse::ok(SystemInfo {
        version: VERSION.to_string(),
        config: SystemConfig {
            bind_addr: state.svc.cfg().bind_addr.clone(),
            bind_port: state.svc.cfg().bind_port,
            quic_bind_port: state.svc.cfg().quic_bind_port,
            kcp_bind_port: state.svc.cfg().kcp_bind_port,
            vhost_http_port: state.svc.cfg().vhost_http_port,
            vhost_https_port: state.svc.cfg().vhost_https_port,
            sub_domain_host: state.svc.cfg().sub_domain_host.clone(),
            tcp_mux: state.svc.cfg().transport.tcp_mux,
            tls_force: state.svc.cfg().transport.tls.force,
            max_pool_count: state.svc.cfg().transport.max_pool_count,
            heartbeat_timeout: state.svc.cfg().transport.heartbeat_timeout,
        },
        status: SystemStatus {
            client_counts: snap
                .clients
                .iter()
                .filter(|c| c.status.is_empty() || c.status == "online")
                .count(),
            total_client_counts: snap.total_client_counts,
            proxy_type_count: snap.proxy_type_count,
            cur_conns: snap.cur_conns,
            total_traffic_in: snap.total_traffic_in,
            total_traffic_out: snap.total_traffic_out,
        },
    }))
}

async fn system_traffic(
    State(state): State<Arc<DashState>>,
    Query(q): Query<TrafficQuery>,
) -> Json<ApiResponse<ProxyTrafficResp>> {
    let hist = state.svc.metrics().server_traffic(traffic_window(&q));
    Json(ApiResponse::ok(traffic_resp(hist)))
}

async fn list_clients(
    State(state): State<Arc<DashState>>,
    Query(q): Query<PageQuery>,
) -> Json<ApiResponse<Page<ClientInfo>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let snap = state.svc.dashboard_snapshot().await;
    let total = snap.clients.len();
    let start = (page - 1).saturating_mul(page_size);
    Json(ApiResponse::ok(Page {
        total, page, page_size,
        items: snap.clients.into_iter().skip(start).take(page_size).collect(),
    }))
}

async fn get_client(
    State(state): State<Arc<DashState>>,
    Path(run_id): Path<String>,
) -> Result<Json<ApiResponse<ClientInfo>>, StatusCode> {
    let run_id = urlencoding_decode(&run_id);
    let snap = state.svc.dashboard_snapshot().await;
    match snap.clients.into_iter().find(|c| c.run_id == run_id) {
        Some(c) => Ok(Json(ApiResponse::ok(c))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn kick_client(
    State(state): State<Arc<DashState>>,
    Path(run_id): Path<String>,
) -> Json<ApiResponse<()>> {
    let run_id = urlencoding_decode(&run_id);
    match state.svc.kick_client(&run_id).await {
        Ok(()) => Json(ApiResponse::ok(())),
        Err(e) => Json(ApiResponse { code: 404, msg: e.to_string(), data: () }),
    }
}

#[derive(Deserialize)]
struct ProxyListQuery {
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: usize,
    #[serde(default, rename = "clientId")]
    client_id: String,
    #[serde(default)]
    q: String,
}

async fn list_proxies(
    State(state): State<Arc<DashState>>,
    Query(q): Query<ProxyListQuery>,
) -> Json<ApiResponse<Page<ProxyInfo>>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 200);
    let snap = state.svc.dashboard_snapshot().await;
    let client_id = q.client_id.trim();
    let needle = q.q.trim().to_ascii_lowercase();
    let filtered: Vec<ProxyInfo> = snap
        .proxies
        .into_iter()
        .filter(|p| client_id.is_empty() || p.client_id == client_id)
        .filter(|p| needle.is_empty() || p.name.to_ascii_lowercase().contains(&needle))
        .collect();
    let total = filtered.len();
    let start = (page - 1).saturating_mul(page_size);
    Json(ApiResponse::ok(Page {
        total, page, page_size,
        items: filtered.into_iter().skip(start).take(page_size).collect(),
    }))
}

async fn proxy_traffic(
    State(state): State<Arc<DashState>>,
    Path(name): Path<String>,
    Query(q): Query<TrafficQuery>,
) -> Result<Json<ApiResponse<ProxyTrafficResp>>, StatusCode> {
    let name = urlencoding_decode(&name);
    match state.svc.metrics().proxy_traffic(&name, traffic_window(&q)) {
        Some(hist) => Ok(Json(ApiResponse::ok(traffic_resp(hist)))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

fn urlencoding_decode(raw: &str) -> String {
    percent_decode(raw).unwrap_or_else(|| raw.to_string())
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let h = from_hex(bytes[i + 1])?;
                let l = from_hex(bytes[i + 2])?;
                out.push((h << 4) | l);
                i += 3;
            }
            b'+' => { out.push(b' '); i += 1; }
            c => { out.push(c); i += 1; }
        }
    }
    String::from_utf8(out).ok()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn load_override(assets_dir: &str, rel: &str) -> Option<Vec<u8>> {
    if assets_dir.trim().is_empty() { return None; }
    let path = safe_join(FsPath::new(assets_dir), rel)?;
    std::fs::read(path).ok()
}

fn safe_join(base: &FsPath, rel: &str) -> Option<PathBuf> {
    let mut out = base.to_path_buf();
    for c in FsPath::new(rel).components() {
        match c {
            Component::Normal(x) => out.push(x),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

fn content_type(path: &str) -> &'static str {
    if path.ends_with(".js") || path.ends_with(".mjs") { "application/javascript; charset=utf-8" }
    else if path.ends_with(".css")   { "text/css; charset=utf-8" }
    else if path.ends_with(".html")  { "text/html; charset=utf-8" }
    else if path.ends_with(".svg")   { "image/svg+xml" }
    else if path.ends_with(".png")   { "image/png" }
    else if path.ends_with(".ico")   { "image/x-icon" }
    else if path.ends_with(".woff2") { "font/woff2" }
    else if path.ends_with(".map")   { "application/json" }
    else { "application/octet-stream" }
}

fn bytes_response(content_type: &'static str, body: Vec<u8>) -> Response {
    (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], body).into_response()
}
