
# ZAMA FILE SYSTEM

Imagine a scenario where a client has a large set of potentially small files {F0, F1, …, Fn} that they want to upload to a server and then delete from their local storage. However, the client needs to be able to download any file later and be confident that the file is correct and hasn't been corrupted (whether in transit or due to tampering by the server).

To achieve this, we have implemented a system consisting of a client, a server, and a Merkle tree. The implementation of the Merkle tree is expected to be done manually rather than using a library, though you can use a library for the underlying hash functions.

The client computes a single Merkle tree root hash and retains it locally after uploading the files to the server and deleting their local copies. When requesting the i-th file Fi, the client also requests a Merkle proof Pi from the server. The client then uses this proof to verify that the file's root hash matches the one it saved previously. If the hashes match, the file is verified to be correct.

## Approach

Upon analyzing the problem description, the solution naturally aligns with concepts from blockchain technology. The project implements a stateless client-server file storage service with a centralized server, leveraging Rust for development. The Merkle tree is structured as a standard full binary tree, providing an efficient way to verify file integrity.

## Usage
Change the `SERVER_IP` in the `docker-compose.yml` and then build the images:

```bash
docker-compose build
```

### Running the Server
```bash
docker-compose up server -d
```

### running the client

```bash
docker-compose run client
```

## Demo

For a visual demonstration of how the application works, please refer to the following asciinema recording.

[![asciinema demo](https://asciinema.org/a/wsL91S2c5I9dJVTicz9D7MW8o.png)](https://asciinema.org/a/wsL91S2c5I9dJVTicz9D7MW8o)


## Next Steps

- **Statefulness**: 

Introduce timestamps to record the exact time when files are uploaded, incorporating this metadata into the Merkle tree. This enhancement allows for improved tracking and verification of files over time, ensuring a more robust audit trail for file uploads.

- **Account Abstraction**:

Support a multi-client file storage service by using a symmetric key to represent each client uniquely. This approach provides a secure method for client identification and authentication, allowing multiple clients to interact with the server while maintaining privacy and security.

- **Server Robustness**: 

Transition from in-memory storage of the Merkle tree to a persistent database solution. This change ensures that the Merkle tree can be reloaded from the database upon server restart, maintaining continuity and reliability in data integrity checks and system operations.


- **Multi-file Retrieval**: 

Optimize the Merkle tree implementation to efficiently handle batch query requests. This enhancement allows the server to process multiple file requests simultaneously, providing encapsulated Merkle proofs that reduce communication overhead and improve overall performance.


- **Support file change by index (Insert/Delete/Update)**:


 Implement support for operations such as insertion, deletion, and updates of small files at specific indices within the Merkle tree. This capability ensures that the tree can dynamically adjust its structure and hash values, similar to the operations performed by a [Merkle Patricia Trie](https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie/#example-trie). 

- **Secure the Communication**:

To enhance the security of data transmission between the client and the server, implement SSL (Secure Sockets Layer) to establish a secure communication channel. This ensures that the data exchanged is encrypted, protecting it from interception and tampering by malicious actors.


