use std::env;
mod merkle;
mod server;

fn show_usage() {
    println!("Usage: merkle-tree <command> [args]");
    println!("Commands:");
    println!("  server -- will start the server");
    println!("  upload <server ip address> -- will upload all files in the current directory to the server");
    println!("  download <server ip address> <index> -- will download the file with the given index from the server");
    println!("  verify <server ip address> <index> -- will verify the downloaded file with the given index from the server");
}

fn upload_files(server_ip: &str, dir: &str) {
    todo!()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() == 2 {
        let command = &args[1];
        match command.as_str() {
            "server" => {
                println!("Starting server...");
                let _ = server::server();
            }
            _ => show_usage(),
        }
    } else if args.len() == 3 {
        let command = &args[1];
        match command.as_str() {
            "upload" => {
                let server_ip = &args[2];
                println!("Uploading files to {}", server_ip);
                upload_files(server_ip, ".");
            }
            "download" => {
                // parse integer from args
                let file_index = args[2].parse::<usize>().unwrap();
                // TODO
            }
            "verify" => {
                let file_index = args[2].parse::<usize>().unwrap();
                // TODO
            }
            _ => show_usage(),
        }
    } else {
        show_usage();
    }

    // let _ = server();
}
