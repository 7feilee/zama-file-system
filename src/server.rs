use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use warp::{Filter, Rejection, Reply};

use crate::merkle;

#[derive(Clone)]
struct ServerState {
    files: Arc<RwLock<Vec<(String, Vec<u8>)>>>,
    merkle_tree: Arc<RwLock<Option<merkle::MerkleTree>>>,
}

#[derive(Serialize, Deserialize)]
struct FileWithProof {
    filename: String,
    content: Vec<u8>,
    proof: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    files: Vec<(String, Vec<u8>)>,
}

#[derive(Error, Debug)]
enum ServerError {
    #[error("File not found")]
    FileNotFound,
    #[error("Merkle tree not initialized")]
    MerkleTreeNotInitialized,
}

impl warp::reject::Reject for ServerError {}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "13337".to_string())
        .parse()?;

    let state = ServerState {
        files: Arc::new(RwLock::new(Vec::new())),
        merkle_tree: Arc::new(RwLock::new(None)),
    };

    let upload = warp::post()
        .and(warp::path("upload"))
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_upload);

    let download_by_index = warp::get()
        .and(warp::path("download"))
        .and(warp::path::param::<usize>())
        .and(with_state(state.clone()))
        .and_then(handle_download_by_index);

    let routes = upload.or(download_by_index).recover(handle_rejection);

    println!("Starting server on 0.0.0.0:{}", port);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
    Ok(())
}

fn with_state(
    state: ServerState,
) -> impl Filter<Extract = (ServerState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

async fn handle_upload(
    upload_request: UploadRequest,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    let mut files = state.files.write().await;
    // !todo Support increment operation.
    files.clear();

    for (filename, content) in upload_request.files {
        files.push((filename.clone(), content));
    }

    let file_contents: Vec<Vec<u8>> = files.iter().map(|(_, content)| content.clone()).collect();
    let merkle_tree = merkle::MerkleTree::new(file_contents);
    *state.merkle_tree.write().await = Some(merkle_tree);

    Ok(warp::reply::with_status(
        "Files uploaded successfully",
        warp::http::StatusCode::OK,
    ))
}

async fn handle_download_by_index(
    index: usize,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    let files = state.files.read().await;
    let merkle_tree = state.merkle_tree.read().await;

    let (filename, content) = files
        .get(index)
        .ok_or_else(|| warp::reject::custom(ServerError::FileNotFound))?;

    let merkle_tree = merkle_tree
        .as_ref()
        .ok_or_else(|| warp::reject::custom(ServerError::MerkleTreeNotInitialized))?;

    let proof = merkle_tree.get_proof(index);

    let file_with_proof = FileWithProof {
        filename: filename.clone(),
        content: content.clone(),
        proof,
    };

    Ok(warp::reply::json(&file_with_proof))
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(ServerError::FileNotFound) = err.find() {
        Ok(warp::reply::with_status(
            "File not found",
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else if let Some(ServerError::MerkleTreeNotInitialized) = err.find() {
        Ok(warp::reply::with_status(
            "Server error: Merkle tree not initialized",
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Internal server error",
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}