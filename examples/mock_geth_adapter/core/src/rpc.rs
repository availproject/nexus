use crate::crypto::{get_multi_proof, SEED};
use avail_rust::{
    kate_recovery::{data::Cell, matrix::Position},
    primitives::kate::{Cells, GProof, GRawScalar},
    H256, SDK, U256,
};
use color_eyre::eyre::{eyre, Result};

// AVAIL TESTNET RPC
pub const AVAIL_RPC_CLIENT: &str = "wss://turing-rpc.avail.so/ws";

// Function taken from :
// https://github.com/availproject/avail-light/blob/2832aaa41cd8c1fe4086bbc4978a63a366a52766/core/src/network/rpc/client.rs#L621
/// To get kate proof for a block hash and positions
pub async fn request_kate_proof(block_hash: H256, positions: &[Position]) -> Result<Vec<Cell>> {
    let rpc_client = rpc_client().await?;
    let cells: Cells = positions
        .iter()
        .map(|p| avail_rust::Cell {
            row: p.row,
            col: p.col as u32,
        })
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| eyre!("Failed to convert to cells"))?;

    let proofs: Vec<(GRawScalar, GProof)> = rpc_client
        .rpc
        .kate
        .query_proof(cells.to_vec(), Some(block_hash))
        .await?;

    let contents = proofs
        .into_iter()
        .map(|(scalar, proof)| concat_content(scalar, proof).expect("TODO"));

    Ok(positions
        .iter()
        .zip(contents)
        .map(|(&position, content)| Cell { position, content })
        .collect::<Vec<_>>())
}

/// To request a kate multi proof for a block hash and positions
pub async fn request_kate_multi_proof(
    block_hash: H256,
    positions: &[Position],
) -> Result<Vec<Cell>> {
    let rpc_client = rpc_client().await?;
    let block_details = rpc_client.rpc.chain.get_block(Some(block_hash)).await?;
    let block_len = rpc_client.rpc.kate.block_length(Some(block_hash)).await?;

    let cells: Cells = positions
        .iter()
        .map(|p| avail_rust::Cell {
            row: p.row,
            col: p.col as u32,
        })
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| eyre!("Failed to convert to cells"))?;

    let mut extrinsics_vec = Vec::new();
    let extrinsics = block_details.block.extrinsics;
    for bytes in extrinsics {
        extrinsics_vec.push(avail_core::AppExtrinsic::from(bytes.0));
    }

    let cells_vec = cells
        .into_iter()
        .map(|cell| (cell.row, cell.col))
        .collect::<Vec<_>>();

    let proofs = get_multi_proof(extrinsics_vec, block_len, SEED, cells_vec)?;

    let contents = proofs
        .into_iter()
        .map(|(scalar, proof)| concat_content_multi_proof(scalar, proof).expect("TODO"));

    let positions = positions.iter().zip(contents);

    let mut cells = Vec::new();

    for (_idx, (position, contents)) in positions.enumerate() {
        for (content_idx, content) in contents.iter().enumerate() {
            cells.push(Cell {
                position: Position {
                    row: position.row,
                    col: content_idx as u16,
                },
                content: *content,
            })
        }
    }

    Ok(cells)
}

fn concat_content(scalar: U256, proof: GProof) -> Result<[u8; 80]> {
    let proof: Vec<u8> = proof.into();
    if proof.len() != 48 {
        return Err(eyre!("Invalid proof length"));
    }

    let mut result = [0u8; 80];
    scalar.to_big_endian(&mut result[48..]);
    result[..48].copy_from_slice(&proof);
    Ok(result)
}

fn concat_content_multi_proof(scalars: Vec<U256>, proof: GProof) -> Result<Vec<[u8; 80]>> {
    let mut result = vec![[0u8; 80]; scalars.len()];

    for (idx, scalar) in scalars.into_iter().enumerate() {
        let proof: Vec<u8> = proof.into();
        if proof.len() != 48 {
            return Err(eyre!("Invalid proof length"));
        }

        scalar.to_big_endian(&mut result[idx][48..]);
        result[idx][..48].copy_from_slice(&proof);
    }

    Ok(result)
}

async fn rpc_client() -> Result<SDK> {
    Ok(SDK::new_insecure(AVAIL_RPC_CLIENT)
        .await
        .expect("Failed to create RPC client"))
}
