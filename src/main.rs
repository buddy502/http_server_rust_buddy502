use std::time::Duration;
use std::env;

use std::{collections::{HashSet, HashMap}, sync::Mutex};
use std::collections::hash_map::Entry;

use actix_files::Files;
use actix_web::{get, post, App, HttpResponse, HttpServer, web, Responder};

use actix_web::cookie::Key;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};

use serde::Deserialize;
use serde_json;
use dotenvy::dotenv;

/*
    claude said the mutex is so there aren't multiple copies being worked on
    the hash set is for multiple users
    web::Data is an Arc which gives workers the ability to have the same list,
    but if a worker is locked then they wait till it's finished

    Users needs this type because the usernames need to be unique
    A mutex makes it so that two threads can't access the same memory
*/
// long ahh type
type UserID  = web::Data<Mutex<HashSet<(String, u16)>>>;
// use String for user, "User#0000" so I can get user and id if needed
type Text    = web::Data<Mutex<HashMap<String, String>>>;

// required to convert JSON to struct
#[derive(Debug, Deserialize)]
struct UserForm {
    id: u16,
    username: String, // username is the full username including id
}

#[derive(Deserialize)]
struct ChatText {
    user: UserForm,
    text: String,
}

fn get_secret_key() -> Key {
    let secret = std::env::var("SESSION_KEY")
        .expect("SESSION_KEY must be set, 'SESSION_SECRET=$(openssl rand -hex 64)'");
    // Key::from requires at least 64 bytes
    Key::from(secret.as_bytes())
}

#[get("/usernames")]
async fn get_usernames(query: web::Query<UserForm>) -> String {
    format!("id: {}, username: {}", query.id, query.username)
}

// type in chat if you're hearing me iynyn
#[post("/chat")]
async fn type_in_chat(chattext: Text, form: web::Form<ChatText>) -> impl Responder {
    let usertext: String = form.text.to_string();
    if usertext.is_empty() {
        return HttpResponse::Ok().finish()
    }

    chattext.lock().unwrap().insert(form.user.username.clone(), usertext);
    // temporary print of user with text

    HttpResponse::Created().body(format!("{}", form.user.username));
    HttpResponse::Created().finish()
}

#[post("/usernames")]
async fn user_login_connection(users: UserID, form: web::Form<UserForm>) -> impl Responder {
    let username = form.username.trim();
    let id = form.id;

    if username.is_empty() || username.contains('#') {
        return HttpResponse::BadRequest().body("invalid username");
    }
    
    if id.to_string().len() != 4 {
        return HttpResponse::BadRequest().body("Invalid length of ID")
    }

    let mut set = users.lock().unwrap();

    if set.insert((username.to_string(), id)) {
        return HttpResponse::Created().finish();
    } else {
        return HttpResponse::Conflict().body("Username already exists"); // never gotta give red error cuz red error bad :)
    }
}

//#[delete("/usernames")]
//async fn delete_user(session: Session) -> HttpResponse {
//    HttpResponse::Ok().finish()
//}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // arguments for different port e.g. cargo run -- 3030
    let args: Vec<String> = env::args().collect();
    let port_browser: u16 = args.get(2)
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // same type as Users so .app_data can ask handlers for it later
    let users: UserID = web::Data::new(Mutex::new(HashSet::new()));

    dotenv().expect(".env file doesn't exist");
    let session_key = get_secret_key();

    // move takes ownership of users so it doesn't dangle after main closes
    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), session_key.clone()))
            .service(get_usernames)
            .app_data(users.clone())
            .service(type_in_chat)
            .service(user_login_connection)
            .service(Files::new("/", "../website").index_file("index.html"))
    })
    .client_request_timeout(Duration::from_secs(60))
    .bind(("127.0.0.1", port_browser))?
        .run()
        .await
}
