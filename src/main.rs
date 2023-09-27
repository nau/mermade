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
    } else if args.len() == 4 && args[1] == "upload" {
        let server_url = &args[2];
        let files_dir = &args[3];
        upload_all_and_delete(server_url, files_dir);
    } else if args.len() == 4 && args[1] == "download" {
        let server_url = &args[2];
        // parse integer from args
        let file_index = args[3].parse::<usize>().unwrap();
        download_verify_file(server_url, file_index);
    } else {
        show_usage();
    }
}
