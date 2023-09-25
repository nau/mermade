use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use env_logger;

use serde::Serialize;
mod merkle;

#[derive(Serialize)]
struct Message {
    message: String,
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().json(Message {
        message: "Hello, world!".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    let server = HttpServer::new(|| App::new().route("/get", web::get().to(hello)));

    log::info!("Starting server at 127.0.0.1:8080");

    server.bind("127.0.0.1:8080")?.run().await
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
