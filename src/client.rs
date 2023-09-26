use crate::merkle::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn upload_files(server_ip: &str, dir: &Path) {
    todo!()
}

pub fn upload_all_and_delete(server_ip: &str, merkle_root_file_path: &Path, files_dir: &Path) {
    store_merkle_root(files_dir, merkle_root_file_path);
    upload_files(server_ip, files_dir);
    delete_files(files_dir);
}

fn delete_files(files_dir: &Path) {
    todo!()
}

fn store_merkle_root(files_dir: &Path, merkle_root_file_path: &Path) -> Result<(), std::io::Error> {
    let files = list_files_in_order(files_dir);
    println!("Files: {:?}", files);
    let mut hashes: Vec<[u8; 32]> = Vec::with_capacity(files.len());
    for file in &files {
        let hash = hash_file_by_path(&file);
        hashes.push(hash);
    }
    let merkle_tree = MerkleTree::from_hashes(hashes);

    println!("Merkle root: {}", hex_hash(merkle_tree.get_merkle_root()));
    // store merkle root to file
    let mut mr_file = File::create(merkle_root_file_path)?;
    mr_file.write_all(merkle_tree.get_merkle_root())?;
    Ok(())
}

fn download_file(server_ip: &str, file_index: usize, dir: &Path) {
    todo!()
}

fn download_proof(server_ip: &str, file_index: usize, dir: &Path) {
    todo!()
}

fn verify_file(server_ip: &str, file_index: usize, dir: &Path) {
    todo!()
}
