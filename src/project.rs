use serde::{Deserialize, Serialize};
use std::fs::{self,File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

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

#[derive(Deserialize, Serialize)]
pub struct Project {
    pub name: Option<String>,
    pub audio: ProjectAudio,
    pub spectrogram: ProjectSpectrogram,
}

#[derive(Deserialize, Serialize)]
pub struct ProjectAudio {
    pub file: String,
    pub channels: usize,
    pub rate: usize,
    pub length: Option<usize>,
}


#[derive(Deserialize, Serialize)]
pub struct ProjectSpectrogram {
    pub samples_per_fft: usize,
    pub samples_per_step: usize,
    pub width: Option<usize>,
    pub height: usize,
}

pub fn get(name: &Path) -> Option<Project> {
    let mut path:PathBuf = PathBuf::from("projects");
    path.push(name);
    path.set_extension("json");
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut project:Project = serde_json::from_reader(reader).ok()?;
    project.name = Some(name.to_str()?.to_owned());
    if let Ok(metadata) = fs::metadata(&format!("projects/{}", project.audio.file)) {
        project.audio.length = Some(metadata.len() as usize / project.audio.channels / 4);
        project.spectrogram.width = Some((project.audio.length.unwrap() - (project.spectrogram.samples_per_fft - project.spectrogram.samples_per_step)) / project.spectrogram.samples_per_step);
    }
    Some(project)
}
