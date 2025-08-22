use std::collections::HashMap;
use std::io;
//use std::iter::Map;

#[derive(Debug, serde::Deserialize)]
pub struct Student {
    id: i16,
    #[serde(rename = "ФИО")]
    name: String,
}

const STUDENT_FILE: &str = "students.tsv";

pub fn read_students() -> csv::Result<HashMap<i16, String>> {
    csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(STUDENT_FILE)?
        .deserialize()
        .map(|res| res.map(|s: Student| (s.id, s.name)))
        .collect()
}

pub fn students_hash() -> Result<String, io::Error> {
    sha256::try_digest(std::path::Path::new(STUDENT_FILE))
}
