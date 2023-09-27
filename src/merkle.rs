use sha2::Digest;
use sha2::Sha256;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;

// We need to get the files in some order to ensure that the merkle root is always the same.
// There are two ways to do this:
// 1. Sort the files by name.
//    TODO: ensure that this is the same on all platforms considering Unicode names (normalization, ordering, etc.)
// 2. Calculate the hash of each file content and sort by hash.
//    This is name-independent approach, but it is slower as it requires two passes over the files.
// Here we use the first approach.
// We read the current working directory and sort the files by name.
pub fn list_files_in_order<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut paths: Vec<_> = fs::read_dir(dir).map(|rd| rd.map(|dir| dir.unwrap()).collect())?;
    paths.sort_by_key(|dir| dir.path());

    for path in paths {
        // we only support regular files, no symlinks, directories, etc.
        match path.metadata() {
            Ok(metadata) if metadata.is_file() => {
                files.push(path.path());
            }
            _ => {}
        }
    }
    Ok(files)
}

pub fn hash_file_by_path<P: AsRef<Path>>(path: P) -> io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    // TODO: use buffered reader if needed
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(hash.into())
}

/// Convert a hash to a hex string.
pub fn hex_hash(hash: &[u8; 32]) -> String {
    hash.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
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
            levels.push(level_hashes);
            level_hashes = next_level_hashes;
            if level_size == 1 {
                levels.push(level_hashes);
                break;
            }
        }
        MerkleTree { levels }
    }

    pub fn size(&self) -> usize {
        self.levels[0].len()
    }

    /// Get the merkle root of the tree.
    pub fn get_merkle_root(&self) -> &[u8; 32] {
        &self.levels.last().unwrap()[0]
    }

    /// Get the merkle proof for the leaf with the given index.
    pub fn make_merkle_proof(&self, index: usize) -> Vec<[u8; 32]> {
        let proof_size = self.levels.len() - 1;
        let hashes_count = self.levels[0].len();
        assert!(index < hashes_count);
        if proof_size == 0 {
            return vec![];
        }
        let mut proof = Vec::with_capacity(proof_size - 1);
        for level in 0..proof_size {
            let level_hashes = &self.levels[level];
            let idx = index / 2usize.pow(level as u32);
            let proof_hash_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            proof.push(level_hashes[proof_hash_idx]);
        }
        proof
    }
}

impl fmt::Display for MerkleTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the formatted representation of the Merkle tree to the provided formatter
        let str = self
            .levels
            .iter()
            .enumerate()
            .map(|(level, hashes)| {
                format!(
                    "Level {}: {:?}",
                    level,
                    hashes.iter().map(hex_hash).collect::<Vec<_>>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", str)
    }
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

/// Calculate merkle root from the hash of the file and the merkle proof.
pub fn calculate_merkle_root_from_proof(
    index: usize,
    hash: &[u8; 32],
    proof: &Vec<[u8; 32]>,
) -> [u8; 32] {
    let mut index = index;
    let mut hasher = Sha256::new();
    let mut hash = hash.clone();
    for sibling in proof {
        if index % 2 == 0 {
            hasher.update(hash);
            hasher.update(sibling);
        } else {
            hasher.update(sibling);
            hasher.update(hash);
        }
        hash = hasher.finalize_reset().into();
        index /= 2;
    }
    hash
}

/// Verify that the merkle root is correct for the given file hash and proof.
pub fn verify_file(
    merkle_root: &[u8; 32],
    file_index: usize,
    file_hash: &[u8; 32],
    proof: &Vec<[u8; 32]>,
) -> Result<(), [u8; 32]> {
    let calculated_merkle_root = calculate_merkle_root_from_proof(file_index, file_hash, proof);
    if calculated_merkle_root != *merkle_root {
        return Err(calculated_merkle_root);
    } else {
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::merkle::*;
    use hex_literal::hex;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn all_proofs_are_valid(hashes in any::<Vec<[u8;32]>>()) {
            // create a merkle tree from random hashes
            let mtree = MerkleTree::from_hashes(hashes.clone());
            // for each hash, generate a proof and verify it
            for (index, hash) in hashes.iter().enumerate() {
              let proof = mtree.make_merkle_proof(index);
              let root = calculate_merkle_root_from_proof(index, hash, &proof);
              // forall hashes, the merkle root from a proof should be the same as the merkle root of the tree
              assert_eq!( root, *mtree.get_merkle_root() );
              // forall hashes, verify_file should return Ok(())
              assert_eq!(
                  verify_file(mtree.get_merkle_root(), index, hash, &proof),
                  Ok(())
              );
          }
        }
    }

    #[test]
    fn merkle_tree_root_on_empty_hashes() {
        let hashes: Vec<[u8; 32]> = Vec::new();
        let mtree = MerkleTree::from_hashes(hashes);
        let root = mtree.get_merkle_root();
        assert_eq!(root, &[0u8; 32])
    }

    #[test]
    fn merkle_tree_root_on_single_hash() {
        let hash: [u8; 32] =
            hex!("1d26c74fd25a4c3dbb09e029fc609588da499fd4af2a41c88f6316c7f8c54cf1");
        let hashes = vec![hash];
        let mtree = MerkleTree::from_hashes(hashes);
        let root = mtree.get_merkle_root();
        assert_eq!(root, &hash)
    }

    #[test]
    fn merkle_tree_root_verify() {
        let hashes = vec![
            hex!("1d26c74fd25a4c3dbb09e029fc609588da499fd4af2a41c88f6316c7f8c54cf1"),
            hex!("44c92e3a70ad3307b7056871c2bdb096d8bfa9373f5bf06a79bb6324a20ff2fb"),
            hex!("fe2d958bad389d6522b04844acc0dced92bcdce95c87971ccbe0f3ad74543f0e"),
            hex!("bbd1319ff740a5546ea65c0d3596672a3705cb9f496012ad6f089e1e0ab6331d"),
            hex!("c5fbbae0208e0c69e6f28fddce5b3770141c405f50100f666dce23c110090345"),
            hex!("dcbccb66ce7ebd666ce5837ce9d73df56049538623e4492ad6b98b37de9751ac"),
        ];
        let mtree = MerkleTree::from_hashes(hashes.clone());
        assert_eq!(
            *mtree.get_merkle_root(),
            hex!("909f4133d05851b483a924b2f3b565651a59efc2ecfcf522c161e446f9638a74")
        );
        for (index, hash) in hashes.iter().enumerate() {
            let proof = mtree.make_merkle_proof(index);
            let root = calculate_merkle_root_from_proof(index, hash, &proof);
            assert_eq!(
                root,
                hex!("909f4133d05851b483a924b2f3b565651a59efc2ecfcf522c161e446f9638a74")
            );
            assert_eq!(
                verify_file(mtree.get_merkle_root(), index, hash, &proof),
                Ok(())
            );
        }
    }
}
