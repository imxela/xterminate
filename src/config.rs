use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub attempt_graceful: bool,
    pub graceful_timeout: u32,
    pub compatibility: Compatibility
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Compatibility {
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32
}