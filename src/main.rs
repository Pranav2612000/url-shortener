#[macro_use]
extern crate rocket;

use rocket::{
    http::Status,
    response::{status, Redirect},
    State,
};
use shuttle_runtime::CustomError;
use sqlx::migrate::Migrator;
use sqlx::{PgPool, FromRow};
use url::Url;
use serde::{Serialize, Deserialize};

struct AppState {
    pool: PgPool,
}

#[derive(Serialize, Deserialize, FromRow)]
struct StoredURL {
    pub id: String,
    pub url: String,
}

#[get("/knockknock")]
fn knockknock() -> &'static str {
    "Who's there?"
}

#[get("/x/urls")]
async fn get_all(state: &State<AppState>) -> Result<String, status::Custom<String>> {
    let stored_urls: Vec<StoredURL> = sqlx::query_as("SELECT * from urls")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| {
            status::Custom(
                Status::InternalServerError,
                "Something went wrong. Please try again".into(),
           )
        })?;

    let response = serde_json::to_string(&stored_urls).unwrap();

    Ok(response)
}

#[get("/<id>")]
async fn redirect(id: String, state: &State<AppState>) -> Result<Redirect, status::Custom<String>> {
    let stored_url: StoredURL = sqlx::query_as("SELECT * from urls where id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => status::Custom(
                Status::NotFound,
                "The requested shortened url does not exist".into(),
            ),
            _ => status::Custom(
                Status::InternalServerError,
                "Something went wrong. Please try again".into(),
           ) 
        })?;

    Ok(Redirect::to(stored_url.url))
}

#[post("/", data = "<url>")]
async fn shorten(url: String, state: &State<AppState>) -> Result<String, status::Custom<String>> {
    let id = &nanoid::nanoid!(6);

    let parsed_url = Url::parse(&url).map_err(|err| {
        status::Custom(
            Status::UnprocessableEntity,
            format!("url validation failed: {err}"),
        )
    })?;

    sqlx::query("INSERT INTO urls(id, url) VALUES ($1, $2)")
        .bind(id)
        .bind(parsed_url.as_str())
        .execute(&state.pool)
        .await
        .map_err(|_| {
            status::Custom(
                Status::InternalServerError,
                "Something went wrong. Please try again".into()
            )
        })?;

    Ok(format!("https://url-reducer.shuttleapp.rs/{id}"))
}

static MIGRATOR: Migrator = sqlx::migrate!("./db");

#[shuttle_runtime::main]
async fn init(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    MIGRATOR.run(&pool).await.map_err(CustomError::new)?;

    let state = AppState { pool };
    let rocket = rocket::build().mount("/", routes![knockknock, shorten, redirect, get_all])
        .manage(state);

    Ok(rocket.into())
}
