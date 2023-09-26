use std::env;
mod client;
mod merkle;
mod server;
use client::*;

fn show_usage() {
    println!("Usage: mermade <command> [args]");
    println!("Commands:");
    println!("  server -- will start the server");
    println!(
        "  upload <server url> -- will upload all files in the current directory to the server, store the merkle root and delete the files"
    );
    println!("  download <server url> <index> -- will download the file with the given index from the server");
    println!("  verify <server url> <index> -- will verify the downloaded file with the given index from the server");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let command = &args[1];
        match command.as_str() {
            "server" => {
                let _ = server::server();
            }
            _ => show_usage(),
        }
    } else if args.len() == 3 {
        let command = &args[1];
        match command.as_str() {
            "upload" => {
                let server_url = &args[2];
                let client = Client::new(server_url, "merkle_root", "files");
                client.upload_all_and_delete();
            }
            _ => show_usage(),
        }
    } else if args.len() == 4 {
        let command = &args[1];
        let server_url = &args[2];
        match command.as_str() {
            "download" => {
                // parse integer from args
                let file_index = args[3].parse::<usize>().unwrap();
                let client = Client::new(server_url, "merkle_root", "files");
                client.download_file(file_index);
            }
            "verify" => {
                let file_index = args[3].parse::<usize>().unwrap();
                let client = Client::new(server_url, "merkle_root", "files");
                client.verify_file(file_index);
            }
            _ => show_usage(),
        }
    } else {
        show_usage();
    }
}
