# Mermade – Merkle Tree Client/Server

## Challenge

Imagine a client has a large set of potentially small files {F0, F1, …, Fn} and wants to upload them to a server and then delete its local copies. The client wants, however, to later download an arbitrary file from the server and be convinced that the file is correct and is not corrupted in any way (in transport, tampered with by the server, etc.).

You should implement the client, the server and a Merkle tree to support the above (we expect you to implement the Merkle tree rather than use a library, but you are free to use a library for the underlying hash functions).

The client must compute a single Merkle tree root hash and keep it on its disk after uploading the files to the server and deleting its local copies. The client can request the i-th file Fi and a Merkle proof Pi for it from the server. The client uses the proof and compares the resulting root hash with the one it persisted before deleting the files - if they match, file is correct.

## Solution

The client is a CLI tool.

```text
Usage: mermade <command> [args]
Commands:
  server <port> -- will start the server on the given port
  upload <server url> <files_dir> -- will upload all files in the <files_dir> directory to the server,
          output the merkle root to STDOUT and delete the files.
          The Merkle Root is written to STDOUT in HEX format.
          Example: mermade upload http://localhost:8080 files > merkle_root.txt

  download <server url> <index> -- will download the file with the given index from the server,
          verify its merkle proof and output the file to stdout.
          The Merkle Root is read from STDIN in HEX format.
          If the merkle proof is invalid, the program will exit with an error code.
          Example: mermade download http://localhost:8080 0 > file.txt < merkle_root.txt
```

To start the server on port 8080, run:

```bash
mermade server 8080
```

The server exposes 3 REST API endpoints:

```text
POST /upload -- accepts a file upload
GET /files/{index} -- returns a file by its index
GET /proofs/{index} -- returns a Merkle proof for a file by its index
```

The server stores all files in a directory named "files" in its current working directory.

On client's first GET request after an upload, the server computes Merkle proofs for each file and stores them in _proof_ files in "proofs" directory.

Each proof file contains a Merkle proof for a file with the same index as the proof file, in binary format.

This is a very simple and efficient solution. The Merkle tree and proofs are computed only once, and proofs are essentially cached. Serving static files is very efficient.

Then, on client GET request, the server simply [`sendfile`](https://linuxgazette.net/issue91/tranter.html) the file and the proof file to the client.

## Merke Tree

I use SHA256 as a hash function. It's fast and secure enough for this purpose.

I compute and store the full Merkle tree in memory in the following form on both client and server.
It's a vector of levels of the tree, where each level is a vector of hashes of the nodes on that level.
So, for 3 files with hashes "aa", "bb", "cc" the tree will look like this:

```javascript
[
    ["aa", "bb", "cc", "cc"],
    ["dd", "ee"],
    ["ff"]
]
```

where "ff" is the Merkle Root.

This is not the most efficient way to store the tree, but it's simple, easy to implement, and it works.
From this implementation it's trivial to derive both Merkle root and proofs.

This implementation requires ~2*32 bytes per file, which is not too bad.

If needed I can implement a "rolling" Merkle root computation, requiring ~2*log2(N) memory, where N is the number of files.

There is a property-based test that verifies that the Merkle tree is correct.
It generates random hashes and verifies that for every Merkle proof the computed Merkle root is the same as computed from the tree.

## How to build

I use Nix Flakes to setup my dev environment. You can use it too, or you can install Rust and Cargo manually.

Run `nix develop` to enter the dev environment.

Run `cargo build --release` to build the project.

## How to run

Run `cargo run -- server 8080` to start the server on port 8080.

Run  `cargo run -- upload http://localhost:8080 files > merkle_root` to upload all files from the "files" directory to the server and store the Merkle Root in the "merkle_root" file.

Run `cargo run -- download http://localhost:8080 0 > file0 < merkle_root` to download the file with index 0 from the server.

## How to run in Docker

Run `docker build -t mermade .` to build the Docker image.

Run `docker run -p 8080:8080 mermade server 8080` to start the server on port 8080.

To upload files, run

```bash
docker run -i -v /absolute/path/to/files:/files mermade upload http://172.17.0.1:8080 /files > merkle_root
```

To download a file, run

```bash
docker run -i mermade download http://172.17.0.1:8080 0 < merkle_root
```
