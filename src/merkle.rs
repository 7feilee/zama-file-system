use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof {
    hashes: Vec<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct MerkleNode {
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
    hash: Vec<u8>,
}

pub struct MerkleTree {
    root: Option<Box<MerkleNode>>,
}

impl MerkleTree {
    pub fn new(data: Vec<Vec<u8>>) -> Self {
        let mut nodes: Vec<MerkleNode> = data
            .into_iter()
            .map(|item| MerkleNode {
                left: None,
                right: None,
                hash: blake3::hash(&item).as_bytes().to_vec(),
            })
            .collect();

        while nodes.len() > 1 {
            let mut new_level: Vec<MerkleNode> = Vec::new();
            for chunk in nodes.chunks(2) {
                if chunk.len() == 2 {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(&chunk[0].hash);
                    hasher.update(&chunk[1].hash);
                    let hash = hasher.finalize().as_bytes().to_vec();
                    new_level.push(MerkleNode {
                        left: Some(Box::new(chunk[0].clone())),
                        right: Some(Box::new(chunk[1].clone())),
                        hash,
                    });
                } else {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(&chunk[0].hash);
                    hasher.update(&chunk[0].hash);
                    let hash = hasher.finalize().as_bytes().to_vec();
                    new_level.push(MerkleNode {
                        left: Some(Box::new(chunk[0].clone())),
                        right: Some(Box::new(chunk[0].clone())),
                        hash,
                    });
                }
            }
            nodes = new_level;
        }

        MerkleTree {
            root: nodes.pop().map(Box::new),
        }
    }

    pub fn depth(&self) -> usize {
        fn node_depth(node: &Option<Box<MerkleNode>>) -> usize {
            match node {
                Some(n) => 1 + usize::max(node_depth(&n.left), node_depth(&n.right)),
                None => 0,
            }
        }
        node_depth(&self.root)
    }

    pub fn root_hash(&self) -> Option<&Vec<u8>> {
        self.root.as_ref().map(|node| &node.hash)
    }

    pub fn get_proof(&self, index: usize) -> Vec<Vec<u8>> {
        let mut proof = Vec::new();
        let mut current_node: &Option<Box<MerkleNode>> = &self.root;

        let bit_idx = format!("{:0depth$b}", index, depth = self.depth() - 1);

        for bit in bit_idx.chars() {
            if let Some(node) = current_node {
                let is_left = bit == '0';
                let sibling = if is_left {
                    node.right.as_ref().map(|n| n.hash.clone())
                } else {
                    node.left.as_ref().map(|n| n.hash.clone())
                };

                if let Some(hash) = sibling {
                    proof.push(hash);
                }

                current_node = if is_left { &node.left } else { &node.right };
            }
        }
        proof
    }
}

pub fn verify_proof(root_hash: &[u8], data: &[u8], proof: &[Vec<u8>], index: usize) -> bool {
    let mut current_hash = blake3::hash(data).as_bytes().to_vec();
    let mut current_index = index;

    let mut proof_copy = proof.to_vec();
    proof_copy.reverse();

    for hash in proof_copy {
        let mut hasher = blake3::Hasher::new();
        let is_current_left = current_index % 2 == 0;

        if is_current_left {
            hasher.update(&current_hash);
            hasher.update(&hash);
        } else {
            hasher.update(&hash);
            hasher.update(&current_hash);
        }

        current_hash = hasher.finalize().as_bytes().to_vec();
        current_index /= 2;
    }
    current_hash == root_hash
}
