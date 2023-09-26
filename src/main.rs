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
    println!("  download <server url> <index> -- will download the file with the given index from the server,
    verify its merkle proof and output the file to stdout.
    If the merkle proof is invalid, the program will exit with an error code.
    ");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "server" {
        let _ = server::server();
    } else if args.len() == 3 && args[1] == "upload" {
        let server_url = &args[2];
        let client = Client::new(server_url, "merkle_root", "files");
        client.upload_all_and_delete();
    } else if args.len() == 4 && args[1] == "download" {
        let server_url = &args[2];
        // parse integer from args
        let file_index = args[3].parse::<usize>().unwrap();
        let client = Client::new(server_url, "merkle_root", "files");
        client.download_verify_file(file_index);
    } else {
        show_usage();
    }
}
