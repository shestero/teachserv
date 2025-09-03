use std::path::PathBuf;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FileRec {
    pub file: PathBuf,
    pub age: u64
}