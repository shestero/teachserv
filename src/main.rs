use std::ffi::OsStr;
use std::fs;
use std::io::{Error, ErrorKind, Result};

use std::time::{SystemTime, SystemTimeError};
use chrono::{Duration, NaiveDate};
use tera::{Tera, Value, from_value};
use timer::Timer;

use config::Config;

use actix_web::{cookie::Key, App, HttpServer};
use actix_web::web::{PayloadConfig, scope};
use actix_identity::IdentityMiddleware;
use actix_session::{storage::RedisSessionStore, SessionMiddleware}; // unused
use actix_session::storage::CookieSessionStore;
use actix_web_httpauth::middleware::HttpAuthentication;

mod routes;
mod teachrec;
mod attendance;
mod filerec;
mod wrong_pwd;

use routes::{index, student, teacher, api_tables};
use crate::filerec::FileRec;

lazy_static::lazy_static! {
    static ref settings: Config = Config::builder()
        .add_source(config::File::with_name("./teachserv"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .expect("teachserv.toml not found!");

    static ref captcha_secret: Vec<u8> =
        settings
            .get_string("captcha_secret")
            .map(|s| s.into_bytes())
            .unwrap_or(b"very-secret-key-change-in-prod-please".to_vec());

    static ref valid_sec: i64 =
        i64::try_from(settings.get_int("valid_sec").unwrap_or(60)).unwrap_or(60);

    static ref cooldown_time: std::time::Duration =
        settings.get_string("cooldown_time")
            .map(|s| humantime::parse_duration(s.as_str()).expect("wrong cooldown_time value: {s}"))
            .unwrap_or(std::time::Duration::from_secs(600)); // default: 10 minutes

    static ref host: String =
        settings.get_string("host").unwrap_or("localhost".to_string());
    static ref port: u16 =
        u16::try_from(settings.get_int("port").unwrap_or(8888)).unwrap_or(8888);

    static ref max_table_age_days: u64 =
        u64::try_from(settings.get_int("max_table_age").unwrap_or(100)).unwrap_or(100);

    // Limit of PUT payload (size of table)
    static ref payload_limit: usize =
        usize::try_from(settings.get_int("payload_limit").unwrap_or(512*1024)).unwrap_or(512*1024);

    static ref api_login: String =
        settings.get_string("api.login").expect("api.login not defined");
    static ref api_password: String =
        settings.get_string("api.password").expect("api.password not defined");
}

fn files_with_age(dir: &str) -> Result<Vec<FileRec>> {
    let now = SystemTime::now();
    Ok(
        fs::read_dir(dir)?
            .map(|entry| {
                let path = entry?.path();
                let extension = path.extension();
                let ok = path.is_file() && (
                    extension == Some(OsStr::new("tsv")) || extension == Some(OsStr::new("bak"))
                );

                Ok(
                    ok.then(|| {
                        let metadata = fs::metadata(&path)?;
                        let modified_time = metadata.modified()?;

                        let elapsed_duration = now.duration_since(modified_time)
                            .map_err(|e: SystemTimeError| {
                                Error::new(ErrorKind::Other, format!("System time error: {}", e))
                            })?; // Handle potential error if time is earlier than modification
                        let days_ago: u64 = elapsed_duration.as_secs() / 3600 / 24;

                        Ok(FileRec { file: path, age: days_ago })
                    })
                )
            })
            .map(|r: Result<_>| r?.transpose())
            .filter_map(|r| r.transpose())
            .collect::<Result<_>>()?
    )
}

fn rm_old_files(dir: &str) {
    match files_with_age(dir) {
        Err(e) =>
            println!("Error during timer attendance check: {}", e),
        Ok(v) =>
            v.iter().for_each(|filerec::FileRec{ file: path, age}|
                if *age >= *max_table_age_days {
                    println!("Too old ({}): {}", age, path.display());
                    if let Err(e) = fs::remove_file(path) {
                        println!("Cannot delete file {}: {}!", path.display(), e);
                    }
                }
            ),
    };
}

fn on_timer() {
    rm_old_files("attendance/inbox/");
    rm_old_files("attendance/outbox/");
}

fn format_date_rus(value: &Value, _: &std::collections::HashMap<String, Value>) -> tera::Result<Value> {
    let date: NaiveDate = from_value(value.clone()).unwrap();
    Ok(tera::to_value(date.format("%d.%m.%Y").to_string()).unwrap())
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Application started.");

    // Start timer
    let timer = Timer::new();
    let _guard = timer.schedule_repeating(Duration::seconds(5), || {
        on_timer();
    });

    // When using `Key::generate()` it is important to initialize outside of the
    // `HttpServer::new` closure. When deployed the secret key should be read from a
    // configuration file or environment variables.
    //let secret_key = Key::generate();

//    let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379")
//        .await
//        .unwrap();

    let mut tera = Tera::new("templates/**/*").unwrap();
    tera.autoescape_on(vec![]);
    tera.register_filter("fmt_date_rus", format_date_rus);

    println!("teachserv: bind to {}:{}", *host, *port);
    HttpServer::new(move || {
        let api = scope("/api")
            .wrap(HttpAuthentication::basic(routes::basic_auth_validator))
            .service(api_tables::attendances)
            .service(api_tables::get_attendance)
            .service(api_tables::put_attendance)
            .service(api_tables::put_attendance_no_hash)
            .service(api_tables::delete_attendance)
            .service(student::put_students)
            .service(student::students_hash)
            .service(student::put_teachers)
            .service(student::teachers_hash);

        App::new()
            .app_data(PayloadConfig::new(*payload_limit))
            .app_data(actix_web::web::Data::new(tera.to_owned()))

            // Install the identity framework first.
            // ??
            // The identity system is built on top of sessions. You must install the session
            // middleware to leverage `actix-identity`. The session middleware must be mounted
            // AFTER the identity middleware: `actix-web` invokes middleware in the OPPOSITE
            // order of registration when it receives an incoming request.
            // ??
            .wrap(IdentityMiddleware::default())
            /*
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            */
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build()
            )
            .service(index::index)
            .service(index::login)
            .service(index::logout)
            .service(index::login_form)
            //.service(index::captcha)
            .service(student::students)
            .service(teacher::table)
            .service(teacher::table_form)
            .service(api)
            .service(
                actix_files::Files::new("/static", "static")
                    // for debug:
                    //.index_file("index.html")
                    //.show_files_listing()
                    .use_last_modified(true)
            )
    })
        .bind(((*host).as_str(), *port))?
        .run()
        .await
}
