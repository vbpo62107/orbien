//! `/api/v1/auth/*` route handlers.

use super::{
    auth::{session_cookie, AuthState},
    DashState,
};
use axum::{
    extract::{Json, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use webauthn_rs::prelude::{
    PublicKeyCredential, RegisterPublicKeyCredential,
};

// ── shared response wrapper ───────────────────────────────────────────────────

#[derive(Serialize)]
struct Resp<T: Serialize> {
    code: u16,
    msg: String,
    data: T,
}

fn ok<T: Serialize>(data: T) -> Json<Resp<T>> {
    Json(Resp { code: 200, msg: String::new(), data })
}

fn err(status: StatusCode, msg: &str) -> Response {
    (status, msg.to_string()).into_response()
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn get_auth(state: &DashState) -> Result<&AuthState, Response> {
    state
        .auth
        .as_deref()
        .ok_or_else(|| err(StatusCode::NOT_IMPLEMENTED, "webauthn not configured"))
}

// ── auth status (public) ───────────────────────────────────────────────────

/// `GET /api/v1/auth/status` — always public (no auth required).
///
/// Returns a small JSON payload the SPA uses to decide which login methods
/// to show.  Example response:
/// ```json
/// { "code": 200, "msg": "", "data": { "webauthn": true, "password": true } }
/// ```
pub async fn auth_status(State(state): State<Arc<DashState>>) -> Response {
    let webauthn_available = state.auth.is_some();
    let password_available = !state.cfg.user.is_empty();
    ok(serde_json::json!({
        "webauthn": webauthn_available,
        "password": password_available,
    }))
    .into_response()
}

// ── password login ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginReq {
    username: String,
    password: String,
}

pub async fn login(
    State(state): State<Arc<DashState>>,
    Json(body): Json<LoginReq>,
) -> Response {
    let ok_creds = body.username == state.cfg.user && body.password == state.cfg.password;
    if !ok_creds {
        return err(StatusCode::UNAUTHORIZED, "invalid credentials");
    }

    let mut res = ok(serde_json::json!({ "username": body.username })).into_response();

    // Always issue a session cookie when AuthState is present; when running in
    // Basic-Auth-only mode we still return 200 so the SPA works, but there is
    // no stateful session to set a cookie for.
    if let Some(auth) = &state.auth {
        let token = auth.create_session(&body.username);
        let cookie = session_cookie(&token, false);
        res.headers_mut().insert(header::SET_COOKIE, cookie);
    }

    res
}

// ── logout ────────────────────────────────────────────────────────────────────

pub async fn logout(
    State(state): State<Arc<DashState>>,
    headers: axum::http::HeaderMap,
) -> Response {
    if let Some(auth) = &state.auth {
        if let Some(token) = super::auth::extract_cookie(&headers, "orbien_session") {
            auth.remove_session(&token);
        }
    }
    let clear = session_cookie("", true);
    let mut res = ok(()).into_response();
    res.headers_mut().insert(header::SET_COOKIE, clear);
    res
}

// ── WebAuthn registration begin ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegBeginReq {
    username: String,
}

pub async fn webauthn_register_begin(
    State(state): State<Arc<DashState>>,
    Json(body): Json<RegBeginReq>,
) -> Response {
    let auth = match get_auth(&state) {
        Ok(a) => a,
        Err(e) => return e,
    };

    let existing: Vec<_> = auth
        .passkeys_for(&body.username)
        .iter()
        .map(|pk| pk.cred_id().clone())
        .collect();

    let user_id = uuid_for_name(&body.username);
    let (challenge, reg_state) = match auth.webauthn.start_passkey_registration(
        user_id,
        &body.username,
        &body.username,
        Some(existing),
    ) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("webauthn reg begin error: {e}");
            return err(StatusCode::INTERNAL_SERVER_ERROR, "registration init failed");
        }
    };

    auth.save_reg_state(&body.username, reg_state);
    ok(challenge).into_response()
}

// ── WebAuthn registration finish ───────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegFinishReq {
    username: String,
    credential: RegisterPublicKeyCredential,
}

pub async fn webauthn_register_finish(
    State(state): State<Arc<DashState>>,
    Json(body): Json<RegFinishReq>,
) -> Response {
    let auth = match get_auth(&state) {
        Ok(a) => a,
        Err(e) => return e,
    };

    let reg_state = match auth.take_reg_state(&body.username) {
        Some(s) => s,
        None => return err(StatusCode::BAD_REQUEST, "no pending registration"),
    };

    match auth
        .webauthn
        .finish_passkey_registration(&body.credential, &reg_state)
    {
        Ok(passkey) => {
            auth.store_passkey(&body.username, passkey);
            ok(()).into_response()
        }
        Err(e) => {
            tracing::warn!("webauthn reg finish error: {e}");
            err(StatusCode::BAD_REQUEST, "registration verification failed")
        }
    }
}

// ── WebAuthn login begin ────────────────────────────────────────────────────────

pub async fn webauthn_login_begin(
    State(state): State<Arc<DashState>>,
) -> Response {
    let auth = match get_auth(&state) {
        Ok(a) => a,
        Err(e) => return e,
    };

    let all_keys = auth.all_passkeys();
    if all_keys.is_empty() {
        return err(StatusCode::BAD_REQUEST, "no registered passkeys");
    }

    let (challenge, auth_state) = match auth.webauthn.start_passkey_authentication(&all_keys) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("webauthn login begin error: {e}");
            return err(StatusCode::INTERNAL_SERVER_ERROR, "authentication init failed");
        }
    };

    let state_key = uuid::Uuid::new_v4().to_string();
    auth.save_auth_state(&state_key, auth_state);

    let mut res = ok(challenge).into_response();
    res.headers_mut().insert(
        header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&format!(
            "orbien_wa_state={state_key}; HttpOnly; Path=/api/v1/auth; SameSite=Strict; Max-Age=120"
        ))
        .unwrap(),
    );
    res
}

// ── WebAuthn login finish ───────────────────────────────────────────────────────

pub async fn webauthn_login_finish(
    State(state): State<Arc<DashState>>,
    req_headers: axum::http::HeaderMap,
    Json(credential): Json<PublicKeyCredential>,
) -> Response {
    let auth = match get_auth(&state) {
        Ok(a) => a,
        Err(e) => return e,
    };

    let state_key = match super::auth::extract_cookie(&req_headers, "orbien_wa_state") {
        Some(k) => k,
        None => return err(StatusCode::BAD_REQUEST, "missing auth state"),
    };

    let auth_state = match auth.take_auth_state(&state_key) {
        Some(s) => s,
        None => return err(StatusCode::BAD_REQUEST, "expired or unknown auth state"),
    };

    match auth.webauthn.finish_passkey_authentication(&credential, &auth_state) {
        Ok(auth_result) => {
            let username = auth
                .apply_auth_result(&auth_result)
                .unwrap_or_else(|| "admin".to_string());

            let token = auth.create_session(&username);
            let cookie = session_cookie(&token, false);
            let clear_wa = axum::http::HeaderValue::from_static(
                "orbien_wa_state=; HttpOnly; Path=/api/v1/auth; Max-Age=0",
            );

            let mut res = ok(serde_json::json!({ "username": username })).into_response();
            res.headers_mut().insert(header::SET_COOKIE, cookie);
            res.headers_mut().append(header::SET_COOKIE, clear_wa);
            res
        }
        Err(e) => {
            tracing::warn!("webauthn login finish error: {e}");
            err(StatusCode::UNAUTHORIZED, "authentication failed")
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn uuid_for_name(name: &str) -> uuid::Uuid {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, name.as_bytes())
}
