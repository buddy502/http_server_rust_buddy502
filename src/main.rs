use std::time::Duration;
use std::env;

use std::{collections::{HashSet, HashMap}, sync::Mutex};
use std::collections::hash_map::Entry;

use actix_files::Files;
use actix_web::{get, post, App, HttpResponse, HttpServer, web, Responder};
use serde::Deserialize;

/*
    claude said the mutex is so there aren't multiple copies being worked on
    the hash set is for multiple users
    web::Data is an Arc which gives workers the ability to have the same list,
    but if a worker is locked then they wait till it's finished

    Users needs this type because the usernames need to be unique
    A mutex makes it so that two threads can't access the same memory
*/
// long ahh type
type UserID  = web::Data<Mutex<HashMap<u16, String>>>;
// use String for user, "User#0000" so I can get user and id if needed
type Text    = web::Data<Mutex<HashMap<String, String>>>;

// required to convert JSON to struct
#[derive(Deserialize)]
struct UserForm {
    id: u16,
    username: String, // username is the full username including id
}

#[derive(Deserialize)]
struct ChatText {
    user: UserForm,
    text: String,
}

// get usernames for chat box
#[get("/usernames/{username}")]
async fn get_usernames(user: UserID, query: web::Query<UserForm>) -> impl Responder {
    HttpResponse::Created().finish()
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
    let full_username: &str = form.username.trim();

    if full_username.is_empty() {
        return HttpResponse::BadRequest().body("username required");
    } else if full_username.split('#').count() >= 2 {
        return HttpResponse::BadRequest().body("Wrong format for username");
    } else if full_username.len() <= 5 {
        // username requires at least a letter, 4 digits, and #
        return HttpResponse::BadRequest().body("username too short");
    }

    // split at # then id is #0000 then take number after #
    let user_len = full_username.len() - 5;
    let (username, user_id) = full_username.split_at(user_len);
    let id: &str = &user_id[1..user_id.len()];

    match id.parse::<u16>() {
        Ok(id) => {
            let mut map = users.lock().unwrap();
            match map.entry(id) {
                Entry::Occupied(_) => HttpResponse::Conflict().body("id already exists"),
                Entry::Vacant(e) => {
                    e.insert(username.to_string());
                    return HttpResponse::Created().body("User added");
                }
            }
        }
        Err(_) => return HttpResponse::BadRequest().body("id must be a number"),
    }
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
