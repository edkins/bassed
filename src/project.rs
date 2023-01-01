use serde::Serialize;
use std::fs;

#[derive(Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProjectStub {
    pub name: String,
}

#[derive(Serialize)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectStub>
}

pub fn list() -> Vec<ProjectStub> {
    let mut result = vec![];
    if let Ok(iter) = fs::read_dir("projects") {
        for dir_entry in iter {
            if let Ok(dir_entry) = dir_entry {
                if dir_entry.file_type().is_ok() && dir_entry.file_type().unwrap().is_file() && dir_entry.path().extension().and_then(|s|s.to_str()) == Some("json") {
                    if let Some(name) = dir_entry.path().file_stem().and_then(|s|s.to_str()) {
                        let name = name.to_owned();
                        result.push(ProjectStub { name });
                    }
                }
            }
        }
    }
    result.sort();
    result
}
