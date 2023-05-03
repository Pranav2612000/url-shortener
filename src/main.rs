#[macro_use]
extern crate rocket;

use rocket::{
    http::Status,
    response::{status},
    State,
};
use shuttle_runtime::CustomError;
use sqlx::migrate::Migrator;
use sqlx::{PgPool};
use url::Url;

struct AppState {
    pool: PgPool,
}

#[get("/knockknock")]
fn knockknock() -> &'static str {
    "Who's there?"
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
    let rocket = rocket::build().mount("/", routes![knockknock, shorten])
        .manage(state);

    Ok(rocket.into())
}
