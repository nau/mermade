use crate::merkle::*;
use reqwest::blocking::multipart;
use sha2::Digest;
use sha2::Sha256;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process;

pub struct Client {
    server_url: String,
    merkle_root_file_path: String,
    files_dir: String,
}

impl Client {
    pub fn new(server_url: &str, merkle_root_file_path: &str, files_dir: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            merkle_root_file_path: merkle_root_file_path.to_string(),
            files_dir: files_dir.to_string(),
        }
    }

    pub fn upload_all_and_delete(&self) {
        let files = list_files_in_order(&self.files_dir).unwrap_or_else(|e| {
            eprintln!("Failed to read files in {}: {}", self.files_dir, e);
            process::exit(1);
        });
        println!("Uploading {} files...", files.len());
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/upload", self.server_url);
        for (index, file) in files.iter().enumerate() {
            println!("Uploading file {}", &file.display());
            // TODO: use buffered reader if needed
            // TODO: read each file only once
            // let form = multipart::Form::new().file("file", file).unwrap();
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
        }
        if let Err(e) = self.store_merkle_root() {
            eprintln!("Failed to store merkle root: {}", e);
            process::exit(1);
        }
        // delete files
        self.delete_files();
    }

    fn delete_files(&self) {
        println!("Deleting files...");
        // I will not delete any files just in case,
        // it's a demo anyways.
    }

    fn store_merkle_root(&self) -> Result<(), std::io::Error> {
        let files = list_files_in_order(&self.files_dir)?;
        println!("Calculating Merkle Root for {} files...", &files.len());
        let mut hashes: Vec<[u8; 32]> = Vec::with_capacity(files.len());
        for file in &files {
            let hash = hash_file_by_path(&file);
            hashes.push(hash);
        }
        let merkle_tree = MerkleTree::from_hashes(hashes);
        // store merkle root to file
        let mut mr_file = File::create(&self.merkle_root_file_path)?;
        mr_file.write_all(merkle_tree.get_merkle_root())?;
        println!(
            "Merkle root {} save to file {}",
            hex_hash(merkle_tree.get_merkle_root()),
            self.merkle_root_file_path
        );
        Ok(())
    }

    fn download_file(&self, file_index: usize) -> Result<actix_web::web::Bytes, reqwest::Error> {
        let url = format!("{}/files/{}", self.server_url, file_index);
        return reqwest::blocking::get(url)?.bytes();
    }

    fn download_proof(&self, file_index: usize) -> Result<actix_web::web::Bytes, reqwest::Error> {
        let url = format!("{}/proofs/{}", self.server_url, file_index);
        return reqwest::blocking::get(url)?.bytes();
    }

    fn get_merkle_root(&self) -> Result<[u8; 32], std::io::Error> {
        let mut file = File::open(&self.merkle_root_file_path)?;
        let mut merkle_root = [0u8; 32];
        file.read_exact(&mut merkle_root)?;
        Ok(merkle_root)
    }

    pub fn download_verify_file(&self, file_index: usize) {
        let bytes = self.download_file(file_index).unwrap();
        let file_hash = Sha256::digest(&bytes);
        let proof_bytes = self.download_proof(file_index).unwrap();
        let proof = deserialize_proof(&proof_bytes).unwrap();
        let merkle_root = self.get_merkle_root().unwrap();
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
}

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
