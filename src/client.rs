use crate::merkle::*;
use indicatif::ProgressBar;
use reqwest::blocking::multipart;
use sha2::Digest;
use sha2::Sha256;
use std::io;
use std::io::Write;
use std::process;

pub fn upload_all_and_delete(server_url: &str, files_dir: &str) {
    let files = list_files_in_order(files_dir).unwrap_or_else(|e| {
        eprintln!("Failed to read files in {}: {}", files_dir, e);
        process::exit(1);
    });
    eprintln!("Uploading {} files...", files.len());
    let bar = ProgressBar::new(files.len() as u64);
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/upload", server_url);
    for (index, file) in files.iter().enumerate() {
        // TODO: use buffered reader if needed
        // TODO: read each file only once
        let file_part = multipart::Part::file(file)
            .map(|p| p.file_name(index.to_string()))
            .unwrap_or_else(|e| {
                eprintln!("Failed to read file {}: {}", file.display(), e);
                process::exit(1);
            });
        let form = multipart::Form::new().part("file", file_part);
        let response = client
            .post(&url)
            .multipart(form)
            .send()
            .unwrap_or_else(|e| {
                eprintln!("Failed to upload file {}: {}", file.display(), e);
                process::exit(1);
            });
        if !response.status().is_success() {
            eprintln!("Failed to upload file. HTTP Response: {:?}", response);
            process::exit(1);
        }
        bar.inc(1);
    }
    bar.finish_and_clear();
    eprintln!("Files uploaded!");
    // this traverses the files again, but it's ok for a demo
    if let Err(e) = output_merkle_root(files_dir) {
        eprintln!("Failed to output merkle root: {}", e);
        process::exit(1);
    }
    // delete files
    delete_files();
}

fn delete_files() {
    eprintln!("Deleting files...");
    // I will not delete any files just in case,
    // it's a demo anyways.
    eprintln!("Joking. I'm not deleting anything, it's a demo!");
}

fn output_merkle_root(files_dir: &str) -> Result<(), std::io::Error> {
    let files = list_files_in_order(files_dir)?;
    let mut hashes: Vec<[u8; 32]> = Vec::with_capacity(files.len());
    for file in &files {
        let hash = hash_file_by_path(&file)?;
        hashes.push(hash);
    }
    let merkle_tree = MerkleTree::from_hashes(hashes);
    eprintln!(
        "Merkle Root for {} files: {}",
        &files.len(),
        hex_hash(merkle_tree.get_merkle_root())
    );
    // write string to stdout
    io::stdout().write_all(hex_hash(merkle_tree.get_merkle_root()).as_bytes())?;
    Ok(())
}

fn download_file(
    server_url: &str,
    file_index: usize,
) -> Result<actix_web::web::Bytes, reqwest::Error> {
    let url = format!("{}/files/{}", server_url, file_index);
    reqwest::blocking::get(url)?.error_for_status()?.bytes()
}

fn download_proof(
    server_url: &str,
    file_index: usize,
) -> Result<actix_web::web::Bytes, reqwest::Error> {
    let url = format!("{}/proofs/{}", server_url, file_index);
    reqwest::blocking::get(url)?.error_for_status()?.bytes()
}

// read Merkle root from stdin
fn get_merkle_root() -> Result<[u8; 32], std::io::Error> {
    let mut merkle_root = [0u8; 32];
    // read a string from stdin
    let mut merkle_root_hex = String::new();
    io::stdin().read_line(&mut merkle_root_hex)?;
    hex::decode_to_slice(merkle_root_hex, &mut merkle_root)
        .expect("Invalid hex string for merkle root");
    Ok(merkle_root)
}

/// Download the file with the given index from the server
/// and verify it with its merkle proof
pub fn download_verify_file(server_url: &str, file_index: usize) {
    let bytes = download_file(server_url, file_index).unwrap_or_else(|e| {
        eprintln!("Failed to download file index {}: {}", file_index, e);
        process::exit(1);
    });
    let file_hash = Sha256::digest(&bytes);
    let proof_bytes = download_proof(server_url, file_index).unwrap_or_else(|e| {
        eprintln!(
            "Failed to download proof for file index {}: {}",
            file_index, e
        );
        process::exit(1);
    });
    let proof = deserialize_proof(&proof_bytes).unwrap();
    let merkle_root = get_merkle_root().unwrap();
    match verify_file(&merkle_root, file_index, file_hash.as_ref(), &proof) {
        Ok(_) => {
            io::stdout().write_all(&bytes).unwrap();
        }
        Err(calculated_merkle_root) => {
            eprintln!("File verification failed");
            eprintln!(
                "Calculated merkle root: {}",
                hex_hash(&calculated_merkle_root)
            );
            eprintln!("Expected merkle root: {}", hex_hash(&merkle_root));
            process::exit(1);
        }
    }
}

/// Deserialize a merkle proof from a byte array.
fn deserialize_proof(proof_bytes: &[u8]) -> Result<Vec<[u8; 32]>, String> {
    if proof_bytes.len() % 32 != 0 {
        return Err(format!(
            "Proof size is not a multiple of 32: {}",
            proof_bytes.len()
        ));
    }
    let mut proof = Vec::<[u8; 32]>::new();
    let mut i = 0;
    while i < proof_bytes.len() {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&proof_bytes[i..i + 32]);
        proof.push(hash);
        i += 32;
    }
    Ok(proof)
}
