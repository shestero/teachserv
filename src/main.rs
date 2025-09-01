use std::ffi::OsStr;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::time::{SystemTime, SystemTimeError};

use config::Config;

use actix_web::{cookie::Key, App, HttpServer};
use actix_web::web::{PayloadConfig, scope};
use actix_identity::IdentityMiddleware;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_session::storage::CookieSessionStore;
use actix_web_httpauth::middleware::HttpAuthentication;

use chrono::Duration;
use timer::Timer;

mod routes;
mod teachrec;
mod attendance;

use routes::{index, student, teacher, api_tables};

#[macro_use]
lazy_static::lazy_static! {
    static ref settings: Config = Config::builder()
        .add_source(config::File::with_name("./teachserv"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .expect("teachserv.toml not found!");

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

fn files_with_age(dir: &str) -> Result<Vec<(PathBuf, u64)>> {
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

                        Ok((path, days_ago))
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
            v.iter().for_each(|(path, age)|
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

#[actix_web::main]
async fn main() -> Result<()> {
    // Start timer
    let timer = Timer::new();
    let _guard = timer.schedule_repeating(Duration::seconds(5), || {
        on_timer();
    });

    // When using `Key::generate()` it is important to initialize outside of the
    // `HttpServer::new` closure. When deployed the secret key should be read from a
    // configuration file or environment variables.
    let secret_key = Key::generate();

//    let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379")
//        .await
//        .unwrap();

    println!("teachserv: bind to {}:{}", *host, *port);
    HttpServer::new(move || {
        let api = scope("/api")
            .wrap(HttpAuthentication::basic(routes::basic_auth_validator))
            .service(api_tables::attendances)
            .service(api_tables::get_attendance)
            .service(api_tables::put_attendance)
            .service(api_tables::put_attendance_no_hash)
            .service(api_tables::delete_attendance)
            .service(student::students_hash);

        App::new()
            .app_data(PayloadConfig::new(*payload_limit))

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
            .service(index::captcha)
            .service(student::students)
            .service(teacher::table)
            .service(teacher::table_form)
            .service(api)
            .service(
                actix_files::Files::new("/static", "static")
                    .index_file("index.html") // todo
                    .show_files_listing() // todo
                    .use_last_modified(true)
            )
    })
        .bind(((*host).as_str(), *port))?
        .run()
        .await
}
