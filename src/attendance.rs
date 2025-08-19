use std::error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use chrono::{Datelike, NaiveDate, Weekday};
use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

#[derive(Clone, Debug, Serialize)]
pub struct Attendance {
    id: String,
    open: bool,
    th_id:  i16,
    th_name: String,
    ss_id: i16,
    ss_name: String,
    date_min: NaiveDate,
    date_max: NaiveDate,
    date_filled: Option<NaiveDate>,
    students: HashMap<i16, (String, Vec<String>)>
}

impl Attendance {
    pub fn read(tsv_file: &str) -> io::Result<Attendance> {
        let file = File::open(tsv_file)?;
        let reader = BufReader::new(file);

        let mut students: HashMap<i16, (String, Vec<String>)> = HashMap::new();
        let mut parameters: HashMap<String, String> = HashMap::new();

        for line_result in reader.lines() {
            line_result?
                .split_once('\t')
                .iter()
                .for_each(|(key, value)| {
                    match key.parse::<i16>() {
                        Ok(st_id) => {
                            let (st_name, attendance_table) =
                                value
                                    .split_once('\t')
                                    .map_or(
                                        (value.to_string(), Vec::new()),
                                        |(value, tail)|
                                                (
                                                    value.to_string(),
                                                    tail
                                                        .split('\t')
                                                        .map(|s| s.to_string())
                                                        .collect()
                                                )
                                    );
                            students.insert(st_id, (st_name, attendance_table));
                        },
                        Err(_) => {
                            parameters.insert(key.to_string(), value.to_string());
                        }
                    }
                });
        }

        let format_str = "%Y-%m-%d";

        let attendance = Attendance {
            id: Path::new(tsv_file).file_stem().unwrap().to_str().unwrap().to_string(),
            open: tsv_file.contains("/open/"),
            th_id: parameters.get("th_id").expect("No th_id!").parse().expect("Cannot parse th_id"),
            th_name: parameters.get("th_name").expect("No th_name!").to_string(),
            ss_id: parameters.get("ss_id").expect("No ss_id!").parse().expect("Cannot parse ss_id"),
            ss_name: parameters.get("ss_name").expect("No ss_name!").to_string(),
            date_min: NaiveDate::parse_from_str(parameters.get("date_min").expect("No date_min!"), format_str).expect("Cannot parse date_min"),
            date_max: NaiveDate::parse_from_str(parameters.get("date_max").expect("No date_max!"), format_str).expect("Cannot parse date_max"),
            date_filled: parameters.get("date_filled").map(|d| NaiveDate ::parse_from_str(d, format_str).expect("Cannot parse date_filled")),
            students: students
        };
        Ok(attendance)
    }

    pub fn date_range(&self) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let mut current_date = self.date_min;
        while current_date <= self.date_max {
            dates.push(current_date);
            current_date = current_date.succ_opt().unwrap(); // Increment the date
        }
        dates
    }

    pub fn attendance_row(&self, st_id: i16) -> Vec<(NaiveDate, i16)> {
        let (_, v) =
            self.students
                .get(&st_id)
                .expect(format!("No student with id {st_id}!").as_str());

        self
            .date_range()
            .iter()
            .enumerate()
            .map(|(idx, &date)|
                (date, v.get(idx).map_or(0, |s| s.parse().unwrap_or(0)))
            )
            .collect()
    }

    pub fn html(&self) -> tera::Result<String> {
        let tera = Tera::new("templates/**/*").unwrap();

        let mut v = self.students
            .clone() // todo ?
            .into_iter()
            .collect::<Vec<_>>();

        v.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // todo

        let table =
            format!(
                "<thead>\n<th>id</th>\n<th>Имя</th>\n{}\n</thead>\n",
                self.date_range()
                    .into_iter()
                    .map(|d| {
                        let weekend = match d.weekday() {
                            Weekday::Sat | Weekday::Sun => " class=\"weekend\"",
                            _ => ""
                        };
                        format!("<th{weekend}>{}</th>", d.day())
                    })
                    .collect::<Vec<_>>()
                    .join("\n") // todo: check if Saturday or Sunday
            ) +
            format!(
                "<tbody>\n{}\n</tbody>\n",
                v
                    .into_iter()
                    .map(|(id, (name, v))|
                        format!(
                            "<tr>\n\t<td class=\"idcol\">{id}</td>\n\t<td class=\"namecol\">{name}</td>\n{}\n</tr>\n",
                            self.date_range()
                                .iter()
                                .enumerate()
                                .map(|(idx, d)| {
                                    let weekend = match d.weekday() {
                                        Weekday::Sat | Weekday::Sun => " class=\"weekend\"",
                                        _ => ""
                                    };
                                    let default = "".to_string();
                                    let v = v.get(idx).unwrap_or(&default);
                                    let v = format!(
                                        "<input name=\"S{id:05}D{d}\" type=\"text\" size=\"1\" value=\"{}\">",
                                        v
                                    );
                                    format!("\t<td{weekend}>{}</td>", v)
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                                .as_str()
                        )
                    )
                    .collect::<Vec<_>>()
                    .join("\n")
            ).as_str();

        let mut context = Context::new();
        context.insert("attendance", self);
        context.insert("table", table.as_str());
        tera.render("attendance.html", &context)
    }
}