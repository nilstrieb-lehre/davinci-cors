use crate::actions::{self, Pool};
use crate::error::ServiceErr;
use crate::handlers::auth::{create_normal_jwt, create_refresh_jwt, Claims};
use crate::models::conversion::IntoDto;
use crate::models::NewUser;
use actix_web::error::BlockingError;
use actix_web::web::ServiceConfig;
use actix_web::web::{block, delete, get, post, put, scope, Data, Json, Path};
use actix_web::HttpResponse;
use diesel::result::DatabaseErrorKind;
use dto::{PostUser, Snowflake, User, UserPostResponse};
use jsonwebtoken::EncodingKey;

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
    cfg.route("/hugo", get().to(get_hugo)).service(
        scope("/users")
            .route("", post().to(create_user))
            .route("/me", get().to(get_own_user))
            .route("/me", put().to(edit_own_user))
            .route("/me", delete().to(delete_own_user))
            .route("/me/link", post().to(link_user_with_discord))
            .route("/discord/{snowflake}", post().to(get_user_by_discord)),
    );
}

async fn get_hugo() -> HttpResponse {
    HttpResponse::Ok().body("Hugo Boss")
}

async fn create_user(body: Json<PostUser>, db: Data<Pool>, key: Data<EncodingKey>) -> HttpResult {
    let user = match block(move || {
        let new_user = NewUser {
            id: uuid::Uuid::new_v4(),
            email: &body.email,
            password: &body.password,
            description: &body.description,
            discord_id: None,
        };

        actions::user::insert_user(&db, new_user)
    })
    .await
    {
        Ok(user) => user,
        Err(BlockingError::Error(ServiceErr::DbActionFailed(
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _),
        ))) => return Err(ServiceErr::Conflict("request/email-exists")),
        other => other?,
    };

    let (token, expires) = create_normal_jwt(user.id, &key)?;
    let refresh_token = create_refresh_jwt(user.id, &key)?;

    Ok(HttpResponse::Created()
        .header("Token", token)
        .header("Refresh-Token", refresh_token)
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
    new_user.id = claims.uid; // always update the own user
    let user = block(move || actions::user::update_user(&db, new_user.into_inner().into()))
        .await?
        .into_dto()?;

    Ok(HttpResponse::Ok().json(user))
}

async fn delete_own_user(claims: Claims, db: Data<Pool>) -> HttpResult {
    let amount = block(move || actions::user::delete_user(&db, claims.uid)).await?;

    Ok(match amount {
        0 => HttpResponse::NotFound().body("User not found"),
        1 => HttpResponse::Ok().body("Deleted user."),
        _ => unreachable!(),
    })
}

async fn link_user_with_discord(claims: Claims, db: Data<Pool>, id: Json<Snowflake>) -> HttpResult {
    let user = block(move || {
        actions::user::set_discord_id_user(&db, claims.uid, Some(&id.into_inner().snowflake))
    })
    .await?
    .into_dto()?;

    Ok(HttpResponse::Ok().json(user))
}

async fn get_user_by_discord(user_id: Path<String>, db: Data<Pool>, claims: Claims) -> HttpResult {
    if !claims.uid.is_nil() {
        return Err(ServiceErr::NotFound); // very secret route
    }

    let user = block(move || actions::user::get_user_by_discord(&db, &user_id))
        .await?
        .into_dto()?;

    Ok(HttpResponse::Ok().json(user))
}
