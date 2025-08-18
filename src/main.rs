use attendance::Attendance;
use config::Config;

mod routes;
mod teachrec;
mod attendance;

use routes::index;

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
}

use actix_web::{cookie::Key, App, HttpServer, HttpResponse};
use actix_identity::IdentityMiddleware;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_session::storage::CookieSessionStore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // When using `Key::generate()` it is important to initialize outside of the
    // `HttpServer::new` closure. When deployed the secret key should be read from a
    // configuration file or environment variables.
    let secret_key = Key::generate();

    let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379")
        .await
        .unwrap();

    let a = Attendance::read("attendance/open/0031-20250815_214834.tsv");
    println!("a={:?}", a);
    //let r = a?.unwrap().attendance_row(287);
    //println!("r={:?}", r);
    println!("HTML:\n{:#?}", a?.html());

    println!("teachserv: bind to {}:{}", *host, *port);
    HttpServer::new(move || {
        App::new()
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
    })
        .bind(((*host).as_str(), *port))?
        .run()
        .await
}
