use std::time::Duration;
use std::env;
use std::io::Read;

use std::net::{TcpListener, TcpStream};
use actix_files::Files;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[get("/users")]
async fn user_login_connection() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // implement the users for browser first
    // testing username creating for browser
    println!("Type logout to logout...");
    print!("Create username: ");
    // listen to all connections, get user id's
    let mut quit: bool = false;
    while !quit {
        for stream in listener.incoming() {
            let mut stream = stream?;
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer)?;
            let input_text = String::from_utf8_lossy(&buffer[..n]);
            if input_text.trim().to_lowercase() ==  "logout" {
                println!("{} logged out", input_text);
                quit = true;
                break;
            }
            HttpResponse::Created().body(format!("created {}", input_text));
        }
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let port_browser: u16 = args.get(2)
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    HttpServer::new(|| {
        App::new()
            .service(user_login_connection)
            .service(Files::new("/", "../website").index_file("index.html"))
    })
    .client_request_timeout(Duration::from_secs(60))
    .bind(("127.0.0.1", port_browser))?
        .run()
        .await
}
