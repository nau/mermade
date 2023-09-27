use crate::merkle::*;
use actix_files;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize)]
struct Message {
    message: String,
}

fn compute_proofs_if_needed() -> Result<()> {
    let proofs_dir = PathBuf::from("proofs");
    if !proofs_dir.exists() {
        println!("Computing proofs...");
        std::fs::create_dir(proofs_dir)?;
        let files = list_files_in_order("files");
        println!("Files: {:?}", files);
        let mut hashes: Vec<[u8; 32]> = Vec::with_capacity(files.len());
        for file in &files {
            let hash = hash_file_by_path(&file);
            hashes.push(hash);
        }
        let merkle_tree = MerkleTree::from_hashes(hashes);
        println!("Merkle root: {}", hex_hash(merkle_tree.get_merkle_root()));
        for file in &files {
            let index = match file
                .file_name()
                .map(|s| s.to_str().map(|s| s.parse::<usize>()))
            {
                Some(Some(Ok(index))) => index,
                _ => {
                    return Err(actix_web::Error::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Invalid filename. Must be an index of the file, but got: {}",
                            file.display()
                        ),
                    )));
                }
            };

            let proof = merkle_tree.make_merkle_proof(index);
            let proof_file_path = PathBuf::from("proofs").join(index.to_string());
            let mut proof_file = File::create(proof_file_path)?;
            // Convert Vec<[u8; 32]> to Vec<u8>
            let flattened: Vec<u8> = proof.into_iter().flatten().collect();
            proof_file.write_all(&flattened)?;
        }
    }
    Ok(())
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, Ralph!".to_string())
}

async fn upload_file(mut payload: Multipart) -> Result<HttpResponse> {
    // create files directory if it doesn't exist
    let files_dir = PathBuf::from("files");
    if !files_dir.exists() {
        std::fs::create_dir(files_dir)?;
    }
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let filename = match content_disposition.get_filename() {
            Some(filename) => filename,
            None => {
                return Ok(HttpResponse::BadRequest()
                    .body("Content-Disposition header is missing filename field"))
            }
        };
        let index = match filename.parse::<usize>() {
            Ok(index) => index,
            Err(_) => {
                return Ok(HttpResponse::BadRequest().body(format!(
                    "Invalid filename. Must be an index of the file, but got: {}",
                    filename
                )));
            }
        };
        let filepath = PathBuf::from("files").join(index.to_string());
        println!("File index {}, path {}", index, filepath.display());

        // File::create is blocking operation, use threadpool
        let mut f = std::fs::File::create(filepath)?;
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f.write_all(&data)?;
        }
        f.sync_all()?;
    }
    Ok(HttpResponse::Ok().finish())
}

#[get("/files/{fileindex}")]
async fn download_file(path: web::Path<String>) -> Result<NamedFile> {
    compute_proofs_if_needed()?;
    let filename = PathBuf::from("files").join(path.into_inner());
    println!("Downloading file {}", filename.display());
    // TODO: ensure that it's impossible to download files outside of the current directory
    let named_file = NamedFile::open(&filename)?;
    Ok(named_file)
}

#[get("/proofs/{fileindex}")]
async fn download_proof(path: web::Path<String>) -> Result<NamedFile> {
    compute_proofs_if_needed()?;
    let file_path = PathBuf::from("proofs").join(path.into_inner());
    println!("Downloading proof {}", file_path.display());
    // TODO: ensure that it's impossible to download files outside of the current directory
    let named_file = NamedFile::open(&file_path)?;
    Ok(named_file)
}

#[actix_web::main]
pub async fn server() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .service(download_file)
            .service(download_proof)
            .route("/upload", web::post().to(upload_file))
            .route("/", web::get().to(hello))
    });

    println!("Starting server at 0.0.0.0:8080");

    server.bind("0.0.0.0:8080")?.run().await
}
