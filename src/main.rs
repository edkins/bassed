#[macro_use] extern crate rocket;
use rocket::fs::FileServer;
use rocket::serde::json::Json;
use crate::project::{ProjectListResponse};

mod project;

#[get("/projects")]
fn projects() -> Json<ProjectListResponse> {
    let projects = project::list();
    let response = ProjectListResponse{ projects };
    Json(response)
}

#[launch]
fn launch() -> _ {
    rocket::build().mount("/api", routes![projects]).mount("/", FileServer::from("www/"))
}
