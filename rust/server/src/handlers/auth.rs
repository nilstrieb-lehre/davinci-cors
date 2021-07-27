use crate::actions::{self, Pool};
use crate::error::ServiceErr;
use crate::handlers::HttpResult;
use actix_web::http::header::Header;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_httpauth::headers::authorization;
use actix_web_httpauth::headers::authorization::Bearer;
use chrono::Utc;
use dto::{LoginResponse, UserLogin};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The claims of the JWT
///
/// *Note: the claim extractor rejects valid JWTs, if their refresh field is set to `true`*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// The expiration date of the token
    pub exp: i64,
    /// The user id
    pub uid: Uuid,
    /// If the field is true, the token can only be used to get another token
    pub refresh: bool,
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/token", web::get().to(refresh_token))
        .route("/login", web::post().to(login))
        .route(
            "/get-bot-token/{JWTSECRET}",
            web::get().to(secret_get_bot_user_token),
        );
}

async fn refresh_token(
    req: HttpRequest,
    e_key: web::Data<EncodingKey>,
    d_key: web::Data<DecodingKey<'static>>,
) -> HttpResult {
    let auth = authorization::Authorization::<Bearer>::parse(&req)
        .map_err(|_| ServiceErr::Unauthorized("auth/no-token"))?;

    let claims = validate_token(auth.into_scheme().token(), &d_key)?;

    if claims.refresh {
        let new_token = create_normal_jwt(claims.uid, &e_key)?;
        Ok(HttpResponse::Ok()
            .header("token", format!("Bearer {}", new_token.0))
            .json(dto::RefreshResponse {
                expires: new_token.1,
            }))
    } else {
        Err(ServiceErr::Unauthorized("auth/wrong-token-kind"))
    }
}

async fn login(
    body: web::Json<UserLogin>,
    db: web::Data<Pool>,
    key: web::Data<EncodingKey>,
) -> HttpResult {
    let user =
        web::block(move || actions::user::validate_user_password(&db, &body.email, &body.password))
            .await?;

    match user {
        Some(user) => {
            let refresh_token = create_refresh_jwt(user.id, &key)?;
            let (token, expires) = create_normal_jwt(user.id, &key)?;
            Ok(HttpResponse::Ok()
                .header("token", format!("Bearer {}", token))
                .header("refresh-token", format!("Bearer {}", refresh_token))
                .json(LoginResponse {
                    userid: user.id,
                    expires,
                }))
        }
        None => Ok(HttpResponse::Forbidden().body("Incorrect email or password")),
    }
}

async fn secret_get_bot_user_token(
    token: web::Path<String>,
    e_key: web::Data<EncodingKey>,
) -> HttpResult {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| ServiceErr::InternalServerError("Secret not found".to_string()))?;

    if *token == secret {
        let uuid = uuid::Uuid::nil();

        Ok(HttpResponse::Ok()
            .header(
                "Token",
                format!(
                    "Bearer {}",
                    create_other_jwt(uuid, &e_key, chrono::Duration::weeks(10000))?
                ),
            )
            .finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

pub fn validate_token(token: &str, key: &DecodingKey) -> Result<Claims, ServiceErr> {
    let decoded = jsonwebtoken::decode::<Claims>(&token, key, &Validation::new(Algorithm::HS512))
        .map_err(|_| ServiceErr::JWTokenError)?
        .claims;

    if decoded.exp < Utc::now().timestamp_millis() {
        Err(ServiceErr::TokenExpiredError)
    } else {
        Ok(decoded)
    }
}

/// Returns the token and the expiration date
/// Create a JWT
pub fn create_normal_jwt(user: Uuid, key: &EncodingKey) -> Result<(String, i64), ServiceErr> {
    let lifetime; // several years, kind of a hack but ok

    // make the token last 24 hours for debugging
    #[cfg(debug_assertions)]
    {
        lifetime = chrono::Duration::hours(24);
    }
    #[cfg(not(debug_assertions))]
    {
        lifetime = chrono::Duration::hours(1);
    }
    create_jwt(user, false, key, lifetime)
}

/// Create a refresh JWT
/// Returns the token and the expiration date
pub fn create_refresh_jwt(user: Uuid, key: &EncodingKey) -> Result<String, ServiceErr> {
    let lifetime = chrono::Duration::weeks(1000); // several years, kind of a hack but ok

    create_jwt(user, true, key, lifetime).map(|(token, _)| token)
}

/// Create a custom expiration date jwt
/// Returns the token and the expiration date
pub fn create_other_jwt(
    user: Uuid,
    key: &EncodingKey,
    time: chrono::Duration,
) -> Result<String, ServiceErr> {
    create_jwt(user, false, key, time).map(|(token, _)| token)
}

fn create_jwt(
    uid: Uuid,
    refresh: bool,
    key: &EncodingKey,
    lifetime: chrono::Duration,
) -> Result<(String, i64), ServiceErr> {
    let exp = Utc::now()
        .checked_add_signed(lifetime)
        .expect("valid timestamp")
        .timestamp_millis();

    let claims = Claims { exp, uid, refresh };

    let header = jsonwebtoken::Header::new(Algorithm::HS512);
    jsonwebtoken::encode(&header, &claims, key)
        .map(|str| (str, exp))
        .map_err(ServiceErr::JWTCreationError)
}
