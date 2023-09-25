use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use env_logger;
use std::env;

use actix_multipart::Multipart;
use serde::Serialize;
mod merkle;
use actix_form_data::{Error, Field, Form, Value};
use futures::{StreamExt, TryStreamExt};
use std::io::Write;

#[derive(Serialize)]
struct Message {
    message: String,
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, Ralph!".to_string())
}

async fn upload_file(mut payload: Multipart) -> impl Responder {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap();
        let index = match filename.parse::<usize>() {
            Ok(index) => index,
            Err(_) => {
                return HttpResponse::BadRequest().body(format!(
                    "Invalid filename. Must be an index of the file, but got: {}",
                    filename
                ));
            }
        };
        let filepath = index.to_string();
        println!("File index {}, path {}", index, filepath);

        // File::create is blocking operation, use threadpool
        let mut f = std::fs::File::create(filepath).unwrap();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f.write_all(&data).unwrap();
        }
        f.sync_all().unwrap();
    }
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn server() -> std::io::Result<()> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    let server = HttpServer::new(|| {
        App::new()
            .route("/upload", web::post().to(upload_file))
            .route("/", web::get().to(hello))
    });

    log::info!("Starting server at 0.0.0.0:8080");

    server.bind("0.0.0.0:8080")?.run().await
}

fn show_usage() {
    println!("Usage: merkle-tree <command> [args]");
    println!("Commands:");
    println!("  server -- will start the server");
    println!("  upload <server ip address> -- will upload all files in the current directory to the server");
    println!("  download <server ip address> <index> -- will download the file with the given index from the server");
    println!("  verify <server ip address> <index> -- will verify the downloaded file with the given index from the server");
}

fn upload_files(server_ip: &str, dir: &str) {
    todo!()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() == 2 {
        let command = &args[1];
        match command.as_str() {
            "server" => {
                println!("Starting server...");
                let _ = server();
            }
            _ => show_usage(),
        }
    } else if args.len() == 3 {
        let command = &args[1];
        match command.as_str() {
            "upload" => {
                let server_ip = &args[2];
                println!("Uploading files to {}", server_ip);
                upload_files(server_ip, ".");
            }
            "download" => {
                // parse integer from args
                let file_index = args[2].parse::<usize>().unwrap();
                // TODO
            }
            "verify" => {
                let file_index = args[2].parse::<usize>().unwrap();
                // TODO
            }
            _ => show_usage(),
        }
    } else {
        show_usage();
    }

    // let _ = server();
}
