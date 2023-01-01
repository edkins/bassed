#[macro_use] extern crate rocket;
use rocket::fs::FileServer;

#[get("/projects")]
fn projects() -> &'static str {
    "Hello world"
}

#[launch]
fn launch() -> _ {
    rocket::build().mount("/api", routes![projects]).mount("/", FileServer::from("www/"))
}
