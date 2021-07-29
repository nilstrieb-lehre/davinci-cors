use crate::actions::{self, Pool};
use crate::error::ServiceErr;
use crate::handlers::auth::{create_normal_jwt, create_refresh_jwt, Claims};
use crate::models;
use crate::models::conversion::IntoDto;
use crate::models::NewUser;
use actix_web::web::{block, delete, get, post, put, scope, Data, Json, Path, Query};
use actix_web::web::{patch, ServiceConfig};
use actix_web::HttpResponse;
use dto::{
    ChangePasswordReq, NotificationQueryParams, NotificationRes, PostUser, SingleSnowflake, User,
    UserPostResponse,
};
use jsonwebtoken::EncodingKey;
use tracing::debug;

mod auth;
mod class;
mod extractors;

pub type HttpResult = Result<HttpResponse, ServiceErr>;

pub fn config(cfg: &mut ServiceConfig) {
    other_config(cfg);
    class::class_config(cfg);
    auth::auth_config(cfg);
}

pub fn other_config(cfg: &mut ServiceConfig) {
    cfg.route("/hugo", get().to(get_hugo))
        .route("/bot/notifications", get().to(get_notifications))
        .service(
            scope("/users")
                .route("", post().to(create_user))
                .route("/me", get().to(get_own_user))
                .route("/me", put().to(edit_own_user))
                .route("/me", delete().to(delete_own_user))
                .route("/me/password", patch().to(change_password))
                .route("/me/link", post().to(link_user_with_discord))
                .route("/discord/{snowflake}", get().to(get_user_by_discord)),
        );
}

async fn get_hugo() -> HttpResponse {
    debug!("Someone got Hugo!");

    HttpResponse::Ok().body("Hugo Boss")
}

async fn create_user(
    mut body: Json<PostUser>,
    db: Data<Pool>,
    key: Data<EncodingKey>,
) -> HttpResult {
    // to make the logging safe - we don't want to leak passwords
    let password = std::mem::replace(&mut body.password, "**********".to_string());

    debug!(?body, "create a user");

    let user = block(move || {
        let new_user = NewUser {
            id: uuid::Uuid::new_v4(),
            email: &body.email,
            password: &password,
            description: &body.description,
            discord_id: None,
            token_version: 1,
        };

        actions::user::insert_user(&db, new_user)
    })
    .await?;

    let (token, expires) = create_normal_jwt(user.id, &key)?;
    let refresh_token = create_refresh_jwt(user.id, &key, 1)?;

    Ok(HttpResponse::Created()
        .header("Token", format!("Bearer {}", token))
        .header("Refresh-Token", format!("Bearer {}", refresh_token))
        .json(UserPostResponse {
            user: User {
                id: user.id,
                email: user.email,
                description: user.description,
                classes: None,
            },
            expires,
        }))
}

async fn get_own_user(claims: Claims, db: Data<Pool>) -> HttpResult {
    debug!(uid = %claims.uid, "get own user");

    let (mut user, classes) = block::<_, _, ServiceErr>(move || {
        let user = actions::user::get_user_by_id(&db, claims.uid)?;
        let classes = actions::class::get_classes_by_user(&db, claims.uid)?;

        Ok((
            user.into_dto()?,
            classes
                .into_iter()
                .map(IntoDto::into_dto)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    })
    .await?;

    user.classes = Some(classes);

    Ok(HttpResponse::Ok().json(user))
}

async fn edit_own_user(claims: Claims, db: Data<Pool>, mut new_user: Json<User>) -> HttpResult {
    debug!(uid = %claims.uid, ?new_user, "edit own user");

    new_user.id = claims.uid; // always update the own user
    let user = block(move || actions::user::update_user(&db, new_user.into_inner().into()))
        .await?
        .into_dto()?;

    Ok(HttpResponse::Ok().json(user))
}

async fn delete_own_user(claims: Claims, db: Data<Pool>) -> HttpResult {
    debug!(uid = %claims.uid, "delete own user 😔 rip");

    let amount = block(move || actions::user::delete_user(&db, claims.uid)).await?;

    Ok(match amount {
        0 => HttpResponse::NotFound().body("User not found"),
        1 => HttpResponse::Ok().body("Deleted user."),
        _ => unreachable!(),
    })
}

async fn change_password(
    claims: Claims,
    db: Data<Pool>,
    e_key: Data<EncodingKey>,
    password: Json<ChangePasswordReq>,
) -> HttpResult {
    debug!(uid = %claims.uid, "change user password");

    let user = block(move || {
        let user = actions::user::get_user_by_id(&db, claims.uid)?;
        let validate =
            actions::user::validate_user_password(&db, &user.email, &password.old_password)?;

        if validate.is_none() {
            return Err(ServiceErr::Unauthorized("wrong-password"));
        }

        actions::user::change_user_password(
            &db,
            models::User {
                id: claims.uid,
                email: "".to_string(),
                password: password.into_inner().password,
                description: "".to_string(),
                discord_id: None,
                token_version: 0,
            },
        )?;

        actions::user::increment_token_version(&db, claims.uid)
    })
    .await?;

    let refresh_token = create_refresh_jwt(user.id, &e_key, user.token_version)?;

    Ok(HttpResponse::Ok()
        .header("Refresh-Token", format!("Bearer {}", refresh_token))
        .json(user.into_dto()?))
}

async fn link_user_with_discord(
    claims: Claims,
    db: Data<Pool>,
    id: Json<SingleSnowflake>,
) -> HttpResult {
    debug!(uid = %claims.uid, ?id, "link own user with discord");

    let snowflake = id.into_inner().snowflake;
    snowflake
        .parse::<u64>()
        .map_err(|_| ServiceErr::BadRequest("invalid-snowflake"))?;

    let user = block(move || actions::user::set_discord_id_user(&db, claims.uid, Some(&snowflake)))
        .await?
        .into_dto()?;

    Ok(HttpResponse::Ok().json(user))
}

async fn get_user_by_discord(user_id: Path<String>, db: Data<Pool>, claims: Claims) -> HttpResult {
    debug!(uid = %claims.uid, ?user_id, "get user by discord");

    if !claims.uid.is_nil() {
        return Err(ServiceErr::Unauthorized("bot-only"));
    }

    let user = block(move || actions::user::get_user_by_discord(&db, &user_id))
        .await?
        .into_dto()?;

    Ok(HttpResponse::Ok().json(user))
}

async fn get_notifications(
    params: Query<NotificationQueryParams>,
    db: Data<Pool>,
    claims: Claims,
) -> HttpResult {
    debug!(?params, "Called get notifications");
    if !claims.uid.is_nil() {
        return Err(ServiceErr::Unauthorized("bot-only"));
    }

    let (time, notifications) = block(move || {
        actions::event::get_notifications(
            &db,
            chrono::NaiveDateTime::from_timestamp(params.since / 1000, 0),
        )
    })
    .await?;

    let notifications = notifications.into_dto()?;

    Ok(HttpResponse::Ok().json(NotificationRes {
        notifications,
        time: time.timestamp_millis(),
    }))
}
