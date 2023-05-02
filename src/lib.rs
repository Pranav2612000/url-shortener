#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};

#[get("/knockknock")]
fn knockknock() -> &'static str {
    "Who's there?"
}

#[shuttle_service::main]
async fn init() -> Result<Rocket<Build>, shuttle_service::Error> {
    let rocket = rocket::build().mount("/", routes![knockknock]);

    Ok(rocket.into())
}
