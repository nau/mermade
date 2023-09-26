use actix_files::NamedFile;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger;
use std::env;

use crate::merkle::*;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Serialize)]
struct Message {
    message: String,
}

fn compute_proofs_if_needed() {
    let proofs_dir = PathBuf::from("proofs");
    if !proofs_dir.exists() {
        println!("Computing proofs...");
        std::fs::create_dir(proofs_dir).unwrap();
        let files = list_files_in_order();
        println!("Files: {:?}", files);
        let mut hashes: Vec<[u8; 32]> = Vec::with_capacity(files.len());
        for file in &files {
            let hash = hash_file_by_path(Path::new(&file));
            hashes.push(hash);
        }
        let merkle_tree = MerkleTree::from_hashes(hashes);
        println!("Merkle root: {}", hex_hash(merkle_tree.get_merkle_root()));
        for file in &files {
            let index = file.parse::<usize>().unwrap();
            let proof = merkle_tree.make_merkle_proof(index);
            let proof_file_path = format!("proofs/{}", index);
            let mut proof_file = File::create(proof_file_path).unwrap();
            // Convert Vec<[u8; 32]> to Vec<u8>
            let flattened: Vec<u8> = proof.into_iter().flatten().collect();
            proof_file.write_all(&flattened).unwrap();
        }
    }
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
    compute_proofs_if_needed();
    let filename = path.into_inner();
    // TODO: ensure that it's impossible to download files outside of the current directory
    let named_file = NamedFile::open(&filename)?;
    Ok(named_file)
}

#[get("/proof/{fileindex}")]
async fn download_proof(path: web::Path<String>) -> Result<NamedFile> {
    println!("Downloading proof {}", path);
    let file_path = PathBuf::from(format!("proofs/{}", path.into_inner()));
    // TODO: ensure that it's impossible to download files outside of the current directory
    let named_file = NamedFile::open(&file_path)?;
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
            .service(download_proof)
            .route("/upload", web::post().to(upload_file))
            // .route("/download", web::get().to(download_file))
            .route("/", web::get().to(hello))
    });

    log::info!("Starting server at 0.0.0.0:8080");

    server.bind("0.0.0.0:8080")?.run().await
}
