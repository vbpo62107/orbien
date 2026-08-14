//! Session-cookie + WebAuthn + password-login authentication layer.
//!
//! ## Session flow
//! 1. Client POSTs `/api/v1/auth/login` (password) **or** completes a
//!    WebAuthn ceremony via `/api/v1/auth/webauthn/login/finish`.
//! 2. Server mints a random 32-byte session token, stores it in `AuthState`,
//!    and sets `Set-Cookie: orbien_session=<token>; HttpOnly; Path=/; SameSite=Strict`.
//! 3. Every subsequent API request carries that cookie.
//!
//! ## Backward compatibility
//! If no session cookie is present the middleware falls back to HTTP Basic Auth
//! so existing integrations keep working without changes.

use crate::dashboard::DashState;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use rand::Rng;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use webauthn_rs::{
    prelude::{
        CreationChallengeResponse, PasskeyAuthentication, PasskeyRegistration,
        PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse,
    },
    Webauthn, WebauthnBuilder,
};

// ── session record ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Session {
    username: String,
    created: Instant,
}

const SESSION_TTL: Duration = Duration::from_secs(8 * 3600); // 8 h
const COOKIE_NAME: &str = "orbien_session";

// ── persisted passkey storage (in-memory for single-node deploy) ──────────────

use webauthn_rs::prelude::Passkey;

// ── public AuthState shared via DashState ────────────────────────────────────

pub struct AuthState {
    /// token → session
    sessions: DashMap<String, Session>,
    /// username → Vec<Passkey>
    passkeys: DashMap<String, Vec<Passkey>>,
    /// pending registration states keyed by username
    reg_states: DashMap<String, PasskeyRegistration>,
    /// pending authentication states keyed by a per-request token
    auth_states: DashMap<String, PasskeyAuthentication>,
    pub webauthn: Webauthn,
}

impl AuthState {
    pub fn new(rp_id: &str, rp_origin: &str) -> anyhow::Result<Self> {
        let origin = url::Url::parse(rp_origin)
            .map_err(|e| anyhow::anyhow!("invalid rp_origin {rp_origin}: {e}"))?;
        let webauthn = WebauthnBuilder::new(rp_id, &origin)?.build()?;
        Ok(Self {
            sessions: DashMap::new(),
            passkeys: DashMap::new(),
            reg_states: DashMap::new(),
            auth_states: DashMap::new(),
            webauthn,
        })
    }

    // ── session helpers ───────────────────────────────────────────────────────

    pub fn create_session(&self, username: &str) -> String {
        let token = random_token();
        self.sessions.insert(
            token.clone(),
            Session {
                username: username.to_string(),
                created: Instant::now(),
            },
        );
        self.evict_expired();
        token
    }

    pub fn validate_session(&self, token: &str) -> Option<String> {
        let entry = self.sessions.get(token)?;
        if entry.created.elapsed() > SESSION_TTL {
            drop(entry);
            self.sessions.remove(token);
            return None;
        }
        Some(entry.username.clone())
    }

    pub fn remove_session(&self, token: &str) {
        self.sessions.remove(token);
    }

    fn evict_expired(&self) {
        self.sessions
            .retain(|_, v| v.created.elapsed() <= SESSION_TTL);
    }

    // ── passkey helpers ───────────────────────────────────────────────────────

    pub fn store_passkey(&self, username: &str, passkey: Passkey) {
        self.passkeys
            .entry(username.to_string())
            .or_default()
            .push(passkey);
    }

    pub fn passkeys_for(&self, username: &str) -> Vec<Passkey> {
        self.passkeys
            .get(username)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn all_passkeys(&self) -> Vec<Passkey> {
        self.passkeys
            .iter()
            .flat_map(|e| e.value().clone())
            .collect()
    }

    pub fn update_passkey(&self, username: &str, updated: &Passkey) {
        if let Some(mut entry) = self.passkeys.get_mut(username) {
            for pk in entry.iter_mut() {
                if pk.cred_id() == updated.cred_id() {
                    *pk = updated.clone();
                }
            }
        }
    }

    // ── pending state helpers ─────────────────────────────────────────────────

    pub fn save_reg_state(&self, username: &str, state: PasskeyRegistration) {
        self.reg_states.insert(username.to_string(), state);
    }

    pub fn take_reg_state(&self, username: &str) -> Option<PasskeyRegistration> {
        self.reg_states.remove(username).map(|(_, v)| v)
    }

    pub fn save_auth_state(&self, key: &str, state: PasskeyAuthentication) {
        self.auth_states.insert(key.to_string(), state);
    }

    pub fn take_auth_state(&self, key: &str) -> Option<PasskeyAuthentication> {
        self.auth_states.remove(key).map(|(_, v)| v)
    }
}

fn random_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

// ── Axum middleware ───────────────────────────────────────────────────────────

/// Checks for a valid session cookie **or** falls back to HTTP Basic Auth.
/// The `/api/v1/auth/*` routes and `/healthz` are always public.
pub async fn auth_middleware(
    State(state): State<Arc<DashState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let path = req.uri().path();

    // Always allow auth endpoints and healthz through
    if path.starts_with("/api/v1/auth/") || path == "/healthz" {
        return Ok(next.run(req).await);
    }

    // Also allow static assets through (JS/CSS/fonts for the login page)
    if !path.starts_with("/api/") {
        return Ok(next.run(req).await);
    }

    // 1. Try session cookie
    if let Some(auth) = &state.auth {
        if let Some(token) = extract_cookie(req.headers(), COOKIE_NAME) {
            if auth.validate_session(&token).is_some() {
                return Ok(next.run(req).await);
            }
        }
    }

    // 2. Fall back to Basic Auth (for backward compatibility)
    if !needs_basic_auth(&state) {
        return Ok(next.run(req).await);
    }
    if basic_auth_ok(&state, req.headers()) {
        return Ok(next.run(req).await);
    }

    // 3. Reject
    let mut res = (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    res.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Basic realm=\"Restricted\""),
    );
    Err(res)
}

// ── cookie helpers ────────────────────────────────────────────────────────────

pub fn extract_cookie(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    let cookie_str = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())?;
    for pair in cookie_str.split(';') {
        let pair = pair.trim();
        if let Some(val) = pair.strip_prefix(&format!("{name}=")) {
            return Some(val.to_string());
        }
    }
    None
}

pub fn session_cookie(token: &str, clear: bool) -> HeaderValue {
    if clear {
        HeaderValue::from_str(&format!(
            "{COOKIE_NAME}=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0"
        ))
        .unwrap()
    } else {
        HeaderValue::from_str(&format!(
            "{COOKIE_NAME}={token}; HttpOnly; Path=/; SameSite=Strict; Max-Age={}",
            SESSION_TTL.as_secs()
        ))
        .unwrap()
    }
}

// ── basic-auth helpers (kept for backward compat) ─────────────────────────────

fn needs_basic_auth(state: &DashState) -> bool {
    !state.cfg.user.is_empty() || !state.cfg.password.is_empty()
}

fn basic_auth_ok(state: &DashState, headers: &axum::http::HeaderMap) -> bool {
    use base64::Engine;
    let Some(h) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Some(b64) = h.strip_prefix("Basic ").or_else(|| h.strip_prefix("basic ")) else {
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
