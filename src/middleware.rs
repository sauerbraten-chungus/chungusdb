use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use jsonwebtoken::{DecodingKey, Validation, decode};
use log::{info, warn};
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
struct Claims {
    exp: usize,
    iat: usize,
    sub: String,
}

pub async fn jwt_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match get_token(&headers) {
        Some(token) => {
            if is_valid_token(token, &state.secret) {
                let response = next.run(request).await;
                Ok(response)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

fn get_token(headers: &HeaderMap) -> Option<&str> {
    let Some(raw_token) = headers.get("Authorization") else {
        warn!("event=authentication_rejected reason=missing_authorization_header");
        return None;
    };

    let Ok(authorization) = raw_token.to_str() else {
        warn!("event=authentication_rejected reason=invalid_authorization_header");
        return None;
    };

    let Some(token) = authorization.strip_prefix("Bearer ") else {
        warn!("event=authentication_rejected reason=invalid_authorization_scheme");
        return None;
    };

    Some(token)
}

fn is_valid_token(token: &str, secret: &str) -> bool {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();
    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            info!(
                "event=authentication_succeeded subject={}",
                token_data.claims.sub
            );
            true
        }
        Err(error) => {
            warn!(
                "event=authentication_rejected reason=invalid_token error={}",
                error
            );
            false
        }
    }
}
