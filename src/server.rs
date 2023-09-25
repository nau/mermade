use actix_files::NamedFile;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger;
use std::env;

use crate::merkle;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

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

#[get("/download/{fileindex}")]
async fn download_file(path: web::Path<String>) -> Result<NamedFile> {
    println!("Downloading file {}", path);
    let filename = path.into_inner();
    // TODO: ensure that it's impossible to download files outside of the current directory
    let named_file = NamedFile::open(&filename)?;
    Ok(named_file)
}

#[actix_web::main]
pub async fn server() -> std::io::Result<()> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    let server = HttpServer::new(|| {
        App::new()
            .service(download_file)
            .route("/upload", web::post().to(upload_file))
            // .route("/download", web::get().to(download_file))
            .route("/", web::get().to(hello))
    });

    log::info!("Starting server at 0.0.0.0:8080");

    server.bind("0.0.0.0:8080")?.run().await
}
