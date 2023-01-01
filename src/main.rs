#[macro_use] extern crate rocket;
use rocket::fs::FileServer;
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use crate::project::{Project, ProjectListResponse};
use std::path::PathBuf;

mod project;
mod spectrogram;

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

#[get("/projects/<name>/spectrogram/<i>")]
fn get_spectrogram(name: PathBuf, i: usize) -> Result<Vec<u8>, NotFound<String>> {
    (if let Some(project) = project::get(&name) {
        let data = spectrogram::get_spectrogram(&project.audio, &project.spectrogram, i).ok_or_else(||NotFound("Unable to load audio data".to_owned()))?;
        Ok(data)
    } else {
        Err(NotFound("Could not locate a valid project at this path".to_owned()))
    }).map_err(|e| NotFound(format!("Audio error: {:?}", e)))
}

#[launch]
fn launch() -> _ {
    rocket::build().mount("/api", routes![list_projects, get_project, get_spectrogram]).mount("/", FileServer::from("www/"))
}
