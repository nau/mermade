use crate::merkle::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;

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

    pub fn upload_files(&self) {
        todo!()
    }

    pub fn upload_all_and_delete(&self) {
        self.store_merkle_root();
        self.upload_files();
        self.delete_files();
    }

    fn delete_files(&self) {
        todo!()
    }

    fn store_merkle_root(&self) -> Result<(), std::io::Error> {
        let files = list_files_in_order(&self.files_dir);
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

    pub fn download_file(
        &self,
        file_index: usize,
    ) -> Result<actix_web::web::Bytes, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/files/{}", self.server_url, file_index);
        return reqwest::blocking::get(url)?.bytes();

        /* match download_file("localhost:8080", file_index) {
            Ok(bytes) => {
                println!("Downloaded file with index {}", file_index);
                let file_path = format!("downloaded_{}", file_index);
                save_file(file_path, &bytes).unwrap();
            }
            Err(e) => {
                println!("Error downloading file: {}", e);
            }
        }
        // TODO */
    }

    pub fn save_file<P: AsRef<Path>>(file_path: P, bytes: &[u8]) -> Result<(), std::io::Error> {
        let mut file = File::create(file_path)?;
        file.write_all(bytes)?;
        Ok(())
    }

    fn download_proof(&self, file_index: usize) {
        todo!()
    }

    pub fn verify_file(&self, file_index: usize) {
        /// Verify that the merkle root is correct for the given file hash and proof.
        pub fn verify_file(
            merkle_root: &[u8; 32],
            file_index: usize,
            file_hash: &[u8; 32],
            proof: &Vec<[u8; 32]>,
        ) -> Result<(), [u8; 32]> {
            let calculated_merkle_root =
                calculate_merkle_root_from_proof(file_index, file_hash, proof);
            if calculated_merkle_root != *merkle_root {
                return Err(calculated_merkle_root);
            } else {
                return Ok(());
            }
        }
    }
}
