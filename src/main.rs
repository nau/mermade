use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use env_logger;

use serde::Serialize;
mod merkle;

#[derive(Serialize)]
struct Message {
    message: String,
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().json(Message {
        message: "Hello, world!".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    let server = HttpServer::new(|| App::new().route("/get", web::get().to(hello)));

    log::info!("Starting server at 127.0.0.1:8080");

    server.bind("127.0.0.1:8080")?.run().await
}
