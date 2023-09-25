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
    use crate::merkle::{
        calculate_merkle_root_naively, hex_hash, make_merkle_proof, show_file_hashes, MerkleTree,
    };
    use hex_literal::hex;

    #[test]
    fn merkle_root_on_empty_hashes() {
        let hashes: Vec<[u8; 32]> = Vec::new();
        let root = calculate_merkle_root_naively(hashes);
        assert_eq!(root, [0u8; 32])
    }

    #[test]
    fn merkle_root_on_single_hash() {
        let hash: [u8; 32] =
            hex!("1d26c74fd25a4c3dbb09e029fc609588da499fd4af2a41c88f6316c7f8c54cf1");
        let hashes = vec![hash];
        let root = calculate_merkle_root_naively(hashes);
        assert_eq!(root, hash)
    }

    #[test]
    fn merkle_root() {
        show_file_hashes();
        // Vec<[u8;32 ]> from hex "aabbccdd"
        let hashes = vec![
            hex!("1d26c74fd25a4c3dbb09e029fc609588da499fd4af2a41c88f6316c7f8c54cf1"),
            hex!("44c92e3a70ad3307b7056871c2bdb096d8bfa9373f5bf06a79bb6324a20ff2fb"),
            hex!("fe2d958bad389d6522b04844acc0dced92bcdce95c87971ccbe0f3ad74543f0e"),
            hex!("bbd1319ff740a5546ea65c0d3596672a3705cb9f496012ad6f089e1e0ab6331d"),
            hex!("c5fbbae0208e0c69e6f28fddce5b3770141c405f50100f666dce23c110090345"),
            hex!("dcbccb66ce7ebd666ce5837ce9d73df56049538623e4492ad6b98b37de9751ac"),
        ];
        let root = calculate_merkle_root_naively(hashes.clone());
        assert_eq!(
            root,
            hex!("909f4133d05851b483a924b2f3b565651a59efc2ecfcf522c161e446f9638a74")
        );
        // make_merkle_proof(&hashes, 0);
    }

    #[test]
    fn merkle_root2() {
        show_file_hashes();
        // Vec<[u8;32 ]> from hex "aabbccdd"
        let hashes = vec![
            hex!("1d26c74fd25a4c3dbb09e029fc609588da499fd4af2a41c88f6316c7f8c54cf1"),
            hex!("44c92e3a70ad3307b7056871c2bdb096d8bfa9373f5bf06a79bb6324a20ff2fb"),
            hex!("fe2d958bad389d6522b04844acc0dced92bcdce95c87971ccbe0f3ad74543f0e"),
            hex!("bbd1319ff740a5546ea65c0d3596672a3705cb9f496012ad6f089e1e0ab6331d"),
            hex!("c5fbbae0208e0c69e6f28fddce5b3770141c405f50100f666dce23c110090345"),
            hex!("dcbccb66ce7ebd666ce5837ce9d73df56049538623e4492ad6b98b37de9751ac"),
        ];
        let mtree = MerkleTree::from_hashes(hashes.clone());
        let root = mtree.get_merkle_root();
        assert_eq!(
            *root,
            hex!("909f4133d05851b483a924b2f3b565651a59efc2ecfcf522c161e446f9638a74")
        );
        let proof = mtree.make_merkle_proof(0);
        assert_eq!(proof.len(), 3);
        mtree.show();
        println!(
            "Proof: {:?}",
            proof.iter().map(hex_hash).collect::<Vec<_>>()
        );
    }
}
