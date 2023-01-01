#[macro_use] extern crate rocket;
use rocket::fs::FileServer;
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use crate::project::{Project, ProjectListResponse};
use std::path::PathBuf;

mod project;

#[get("/projects")]
fn list_projects() -> Json<ProjectListResponse> {
    let projects = project::list();
    let response = ProjectListResponse{ projects };
    Json(response)
}

#[get("/projects/<name>")]
fn get_project(name: PathBuf) -> Result<Json<Project>, NotFound<String>> {
    if let Some(project) = project::get(&name) {
        Ok(Json(project))
    } else {
        Err(NotFound("Could not locate a valid project at this path".to_owned()))
    }
}

#[launch]
fn launch() -> _ {
    rocket::build().mount("/api", routes![list_projects, get_project]).mount("/", FileServer::from("www/"))
}
