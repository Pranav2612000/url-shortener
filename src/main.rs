#[macro_use]
extern crate rocket;

use shuttle_runtime::CustomError;
use sqlx::migrate::Migrator;
use sqlx::{PgPool};

#[get("/knockknock")]
fn knockknock() -> &'static str {
    "Who's there?"
}

static MIGRATOR: Migrator = sqlx::migrate!("./db");

#[shuttle_runtime::main]
async fn init(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    MIGRATOR.run(&pool).await.map_err(CustomError::new)?;

    let rocket = rocket::build().mount("/", routes![knockknock]);

    Ok(rocket.into())
}
