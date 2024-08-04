use crate::merkle;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self};
use std::io::{self, Write};
use std::path::{PathBuf};
use thiserror::Error;

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    files: Vec<(String, Vec<u8>)>,
}

#[derive(Serialize, Deserialize)]
struct FileWithProof {
    filename: String,
    content: Vec<u8>,
    proof: Vec<Vec<u8>>,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Environment variable not found: {0}")]
    EnvVar(#[from] env::VarError),
    #[error("No files found in the upload directory")]
    NoFiles,
    #[error("Failed to verify file")]
    VerificationFailed,
}

pub async fn run() -> Result<(), ClientError> {
    dotenv().ok();
    let server_address = format!(
        "http://{}:{}",
        env::var("SERVER_IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
        env::var("PORT").unwrap_or_else(|_| "13337".to_string())
    );
    let folder_path = "db/uploads";

    let files = read_files_from_folder(folder_path)?;
    if files.is_empty() {
        return Err(ClientError::NoFiles);
    }

    let file_contents: Vec<Vec<u8>> = files.iter().map(|(_, content)| content.clone()).collect();
    upload_files(&server_address, files).await?;

    let merkle_tree = merkle::MerkleTree::new(file_contents);
    save_merkle_root(merkle_tree.root_hash().expect("Merkle tree should have a root"))?;
    delete_local_files(folder_path)?;

    loop {
        let file_index = prompt_for_file_index()?;
        let file_with_proof = request_file_with_proof(&server_address, file_index).await?;
        let stored_merkle_root = load_merkle_root()?;

        if merkle::verify_proof(
            &stored_merkle_root,
            &file_with_proof.content,
            &file_with_proof.proof,
            file_index,
        ) {
            println!("File is verified and correct");
            save_file(&file_with_proof.filename, &file_with_proof.content)?;
            println!("File saved successfully");
        } else {
            return Err(ClientError::VerificationFailed);
        }
    }
}

async fn upload_files(
    server_addr: &str,
    files: Vec<(String, Vec<u8>)>,
) -> Result<(), ClientError> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/upload", server_addr))
        .json(&UploadRequest { files })
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(ClientError::Network(format!("HTTP error: {}", response.status())))
    }
}

async fn request_file_with_proof(
    server_addr: &str,
    index: usize,
) -> Result<FileWithProof, ClientError> {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/download/{}", server_addr, index))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(ClientError::Network(format!("HTTP error: {}", response.status())))
    }
}

fn read_files_from_folder(folder_path: &str) -> Result<Vec<(String, Vec<u8>)>, ClientError> {
    let mut files_content = Vec::new();
    let entries = fs::read_dir(folder_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let content = fs::read(&path)?;
            files_content.push((filename, content));
        }
    }

    Ok(files_content)
}

fn save_merkle_root(root: &[u8]) -> Result<(), ClientError> {
    fs::write("merkle_root.txt", root)?;
    Ok(())
}

fn load_merkle_root() -> Result<Vec<u8>, ClientError> {
    Ok(fs::read("merkle_root.txt")?)
}

fn delete_local_files(folder_path: &str) -> Result<(), ClientError> {
    for entry in fs::read_dir(folder_path)? {
        let path = entry?.path();
        if path.is_file() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn prompt_for_file_index() -> Result<usize, ClientError> {
    print!("Enter the index of the file to retrieve: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().parse().map_err(|_| ClientError::Io(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid index",
    )))?)
}

fn save_file(filename: &str, content: &[u8]) -> Result<(), ClientError> {
    let mut path = PathBuf::from("db/downloads");

    path.push(filename);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}