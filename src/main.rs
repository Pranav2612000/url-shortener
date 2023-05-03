#[macro_use]
extern crate rocket;

use sqlx::{PgPool};

#[get("/knockknock")]
fn knockknock() -> &'static str {
    "Who's there?"
}

#[shuttle_runtime::main]
async fn init(#[shuttle_shared_db::Postgres] _pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![knockknock]);

    Ok(rocket.into())
}
