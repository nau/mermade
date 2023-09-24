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
    use crate::merkle::{calculate_merkle_root_naively, make_merkle_proof, show_file_hashes};
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
            hex!("006395992527536bb1f4f9896133f7332de2fa084b7caaf125d1566ad849ccbb"),
            hex!("35123422729b43b1ec55af2978db6602aea5f9f5d3605bc898f726f7a847d3b3"),
            hex!("c5fbbae0208e0c69e6f28fddce5b3770141c405f50100f666dce23c110090345"),
            hex!("dcbccb66ce7ebd666ce5837ce9d73df56049538623e4492ad6b98b37de9751ac"),
        ];
        let root = calculate_merkle_root_naively(hashes.clone());
        assert_eq!(
            root,
            hex!("b81026c419837081d0cab8ad718aab13c47331cb701306d9b291adb038d7a7f1")
        );
        make_merkle_proof(&hashes, 0);
    }
}
