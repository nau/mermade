use sha2::digest::typenum::Log2;
use sha2::Digest;
use sha2::Sha256;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
// We need to get the files in some order to ensure that the merkle root is always the same.
// There are two ways to do this:
// 1. Sort the files by name.
//    TODO: ensure that this is the same on all platforms considering Unicode names (normalization, ordering, etc.)
// 2. Calculate the hash of each file content and sort by hash.
//    This is name-independent approach, but it is slower as it requires two passes over the files.
// Here we use the first approach.
// We read the current working directory and sort the files by name.
pub fn list_files_in_order() -> Vec<String> {
    let mut files = Vec::new();
    let mut paths: Vec<_> = fs::read_dir(".").unwrap().map(|r| r.unwrap()).collect();
    paths.sort_by_key(|dir| dir.path());

    for path in paths {
        let metadata = path.metadata().unwrap();
        // we only support regular files, no symlinks, directories, etc.
        if metadata.is_file() {
            files.push(path.path().display().to_string());
        }
    }
    println!("{:?}", files);
    files
}

pub fn hash_file_by_path(path: &Path) -> [u8; 32] {
    let mut file = File::open(path).unwrap();
    let mut hasher = Sha256::new();
    // TODO: use buffered reader if needed
    io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize();
    format!("{:x}", hash);
    hash.into()
}

pub fn hex_hash(hash: &[u8; 32]) -> String {
    hash.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

pub fn show_file_hashes() {
    let files = list_files_in_order();
    let mut hashes: Vec<[u8; 32]> = Vec::with_capacity(files.len());
    for file in files {
        let hash = hash_file_by_path(Path::new(&file));
        hashes.push(hash);
        let hex_string = hex_hash(&hash);
        println!("{}: {}", file, hex_string);
    }
    let merkle_root = calculate_merkle_root_naively(hashes);
    println!("Merkle root: {}", hex_hash(&merkle_root));
}

pub struct MerkleTree {
    levels: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    // This is a naive and simple implementation of the merkle tree calculation.
    // It requires all the files' hashes in memory.
    // Each hash is 32 bytes, so even for millions of files it is not a huge problem though.
    // The whole tree is stored in memory and it should be up to 2*32*n bytes, where n is the number of files.

    // This approach gives us ability to reuse the Merkle tree for
    // both Merkle Root calculation and Merkle Proof generation.
    pub fn from_hashes(hashes: Vec<[u8; 32]>) -> Self {
        let mut levels = Vec::<Vec<[u8; 32]>>::new();

        if hashes.len() == 0 {
            levels.push(vec![[0u8; 32]]);
            return MerkleTree { levels };
        }

        if hashes.len() == 1 {
            levels.push(hashes);
            return MerkleTree { levels };
        }
        let mut level_hashes = hashes;
        loop {
            let next_level_hashes = calculate_merkle_tree_level(&mut level_hashes);
            let level_size = next_level_hashes.len();
            println!("Level size: {}", level_size);
            levels.push(level_hashes);
            level_hashes = next_level_hashes;
            if level_size == 1 {
                levels.push(level_hashes);
                break;
            }
        }
        MerkleTree { levels }
    }

    pub fn get_merkle_root(&self) -> &[u8; 32] {
        &self.levels.last().unwrap()[0]
    }

    pub fn make_merkle_proof(&self, index: usize) -> Vec<[u8; 32]> {
        let proof_size = self.levels.len() - 1;
        let hashes_count = self.levels[0].len();
        assert!(index < hashes_count);
        if proof_size == 0 {
            return vec![];
        }
        println!(
            "Proof size of {} for hashes {}",
            proof_size,
            self.levels[0].len()
        );
        let mut proof = Vec::with_capacity(proof_size - 1);
        for level in 0..proof_size {
            let level_hashes = &self.levels[level];
            let idx = index / 2usize.pow(level as u32);
            let proof_hash_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            println!(
                "Level size: {}, idx: {}, proof_hash_idx: {}, proof: {}",
                level_hashes.len(),
                idx,
                proof_hash_idx,
                hex_hash(&level_hashes[proof_hash_idx])
            );
            proof.push(level_hashes[proof_hash_idx]);
        }
        proof
    }

    pub fn show(&self) {
        for (level, hashes) in self.levels.iter().enumerate() {
            println!(
                "Level {}: {:?}",
                level,
                hashes.iter().map(hex_hash).collect::<Vec<_>>()
            );
        }
    }
}

// This is a naive and simple implementation of the merkle root calculation.
// It requires all the files' hashes in memory.
// Each hash is 32 bytes, so even for millions of files it is not a huge problem though.
// The implementation reuses the hashes vector to avoid allocating new memory.
// It is recursive, but again it's not a problem as the call stack grows logarithmically.
pub fn calculate_merkle_root_naively(hashes: Vec<[u8; 32]>) -> [u8; 32] {
    let mut hashes = hashes;
    if hashes.len() == 0 {
        return [0; 32];
    }
    let mut hasher = Sha256::new();
    while hashes.len() > 1 {
        // duplicate last element if odd number of elements
        // there is a potential problem, see https://github.com/bitcoin/bitcoin/blob/master/src/consensus/merkle.cpp#L8
        // but it is not a problem for us as we upload client's files,
        // so the client can only harm himself and not the whole system.
        if hashes.len() % 2 == 1 {
            hashes.push(hashes.last().unwrap().clone());
        }
        for i in (0..hashes.len()).step_by(2) {
            hasher.update(hashes[i]);
            hasher.update(hashes[i + 1]);
            let hash = hasher.finalize_reset().into();
            /* println!(
                "Inner Hash of {} and {}: {:?}",
                hex_hash(&hashes[i]),
                hex_hash(&hashes[i + 1]),
                hex_hash(&hash)
            ); */
            hashes[i / 2] = hash;
        }
        hashes.truncate(hashes.len() / 2);
    }
    return hashes[0];
}

fn calculate_merkle_tree_level(hashes: &mut Vec<[u8; 32]>) -> Vec<[u8; 32]> {
    let mut hasher = Sha256::new();
    let mut level_hashes = Vec::with_capacity(hashes.len() / 2);
    // duplicate last element if odd number of elements
    // there is a potential problem, see https://github.com/bitcoin/bitcoin/blob/master/src/consensus/merkle.cpp#L8
    // but it is not a problem for us as we upload client's files,
    // so the client can only harm himself and not the whole system.
    if hashes.len() % 2 == 1 {
        hashes.push(hashes.last().unwrap().clone());
    }
    for i in (0..hashes.len()).step_by(2) {
        hasher.update(hashes[i]);
        hasher.update(hashes[i + 1]);
        let hash = hasher.finalize_reset().into();
        level_hashes.push(hash);
    }
    level_hashes
}

pub fn make_merkle_proof(hashes: &Vec<[u8; 32]>, file_index: usize) -> Vec<[u8; 32]> {
    let size = hashes.len();
    assert!(file_index < size);
    if size == 0 {
        return vec![[0; 32]];
    }
    if size == 1 {
        return hashes.clone();
    }
    let proof_size = (size as f64).log2().ceil() as usize;
    println!("Proof size of {} hashes: {}", size, proof_size);
    let mut proof = Vec::with_capacity(proof_size);
    if file_index % 2 == 0 {
        proof.push(hashes[file_index + 1]);
    } else {
        proof.push(hashes[file_index - 1]);
    };

    proof
}

/*
                   A               A
                 /  \            /   \
               B     C         B       C
              / \    |        / \     / \
             D   E   F       D   E   F   F
            / \ / \ / \     / \ / \ / \ / \
            1 2 3 4 5 6     1 2 3 4 5 6 5 6
*/
pub fn verify_merkle_root(merkle_root: &[u8; 32], proof: &Vec<[u8; 32]>) -> Result<(), [u8; 32]> {
    Result::Ok(())
}
// Merkle tree implementation
