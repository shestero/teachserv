use std::{fs::File, ops::Deref};
use serde::Deserialize;
use crate::routes::login::Login;

#[derive(Debug, Deserialize)]
pub struct TeachRec {
    id: i16,
    name: String,
    pw: String,
}

impl TeachRec {
    pub fn find(login: Login) -> Option<TeachRec> {
        let th_id: i16 = login.login.parse().ok()?;

        let file = File::open("tpws.tsv").expect("No tpws.tsv file!!");
        csv::ReaderBuilder::new()
            .delimiter(b'\t') // Specify tab as the delimiter
            .from_reader(file)
            .deserialize()
            .find_map(|rec|
                {
                    rec.ok()
                        .filter(|rec: &TeachRec| rec.id == th_id && login.check_password(&rec.pw))
                }
            )
    }
    
    pub fn id_and_name(&self) -> String {
        format!("{}\t{}", self.id, self.name)
    }
    
    pub fn split_id_and_name(id_and_name: String) -> (String, String) {
        id_and_name
            .split_once('\t')
            .map_or(
                (String::new(), id_and_name.clone()),
                |(id, name)| (id.to_string(), name.to_string())
            )
    }
}

