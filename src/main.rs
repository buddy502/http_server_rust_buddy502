use std::time::Duration;
use std::env;
use std::io::Read;

use std::{collections::HashSet, sync::Mutex};

use actix_files::Files;
use actix_web::{get, post, App, HttpResponse, HttpServer, web, Responder, http::header};
use serde::Deserialize;

// claude said the mutex is so there aren't multiple copies being worked on
// the hash set is for multiple users
// web::Data is an Arc which gives workers the ability to have the same list,
// but if a worker is locked then they wait till it's finished
type Users = web::Data<Mutex<HashSet<String>>>;

// required to convert JSON to struct
#[derive(Deserialize)]
struct UserForm {
    username: String,
}

#[post("/usernames")]
async fn user_login_connection(users: Users, form: web::Form<UserForm>) -> impl Responder {
    let username: String = form.username.trim().to_string();

    if username.is_empty() {
        return HttpResponse::BadRequest().body("username required");
    }

    users.lock().unwrap().insert(username);
    HttpResponse::Created().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let port_browser: u16 = args.get(2)
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // same type as Users so .app_data can ask handlers for it later
    let users = web::Data::new(Mutex::new(HashSet::<String>::new()));

    // move takes ownership of users so it doesn't dangle after main closes
    HttpServer::new(move || {
        App::new()
            .app_data(users.clone())
            .service(user_login_connection)
            .service(Files::new("/", "../website").index_file("index.html"))
    })
    .client_request_timeout(Duration::from_secs(60))
    .bind(("127.0.0.1", port_browser))?
        .run()
        .await
}
