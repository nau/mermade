use std::env;
mod client;
mod merkle;
mod server;
use client::*;

fn show_usage() {
    println!("Usage: mermade <command> [args]");
    println!("Commands:");
    println!("  server <port> -- will start the server on the given port");
    println!("  upload <server url> <files_dir> -- will upload all files in the <files_dir> directory to the server,
          output the merkle root to STDOUT and delete the files.
          The Merkle Root is written to STDOUT in HEX format.
          Example: mermade upload http://localhost:8080 files > merkle_root.txt
    ");
    println!("  download <server url> <index> -- will download the file with the given index from the server,
          verify its merkle proof and output the file to stdout.
          The Merkle Root is read from STDIN in HEX format.
          If the merkle proof is invalid, the program will exit with an error code.
          Example: mermade download http://localhost:8080 0 > file.txt < merkle_root.txt
    ");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[1] == "server" {
        let _ = server::server(&args[2]);
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
