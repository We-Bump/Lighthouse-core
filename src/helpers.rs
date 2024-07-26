use cosmwasm_std::{ DepsMut, StdError, StdResult };
use data_encoding::HEXLOWER;
use sha3::{ Digest, Keccak256 };
use bech32::{decode, FromBase32};
use crate::{msg::EvmQueryWrapper, state::{ MintGroup, PaymentType }};

pub fn create_group_key(addr: &str, collection_addr: &str, group_name: &str) -> String {
    format!("{}_{}_{}", addr, collection_addr, group_name)
}

pub fn create_global_mint_info_key(collection_addr: &str, group_name: &str) -> String {
    format!("{}_{}", collection_addr, group_name)
}

pub fn create_mint_log_key(collection_addr: &str, token_id: &str) -> String {
    format!("{}_{}", collection_addr, token_id)
}

pub fn create_mint_log_key_404(collection_addr: &str, group_name: &str, order: &str) -> String {
    format!("{}_{}_{}", collection_addr, group_name, order)
}

pub fn create_gated_mint_log_key(
    collection_addr: &str,
    contract_address: &str,
    group_name: &str,
    gated_token_id: &str
) -> String {
    format!("{}_{}_{}_{}", collection_addr, contract_address, group_name, gated_token_id)
}

pub fn validate_merkle_proof(proof: Vec<Vec<u8>>, root: Vec<u8>, leaf: Vec<u8>) -> bool {
    let mut hash = leaf;
    for proof_hash in proof.into_iter() {
        let mut hasher = Keccak256::new();

        // Hash current hash and proof hash together
        // The smaller one goes first
        if hash < proof_hash {
            hasher.update(&hash);
            hasher.update(&proof_hash);
        } else {
            hasher.update(&proof_hash);
            hasher.update(&hash);
        }

        hash = hasher.finalize().to_vec();
    }

    hash == root
}

pub fn pad_address_to_bytes32(address: &str) -> StdResult<Vec<u8>> {
    let trimmed_address = address.trim_start_matches("0x").to_lowercase();

    let decoded = HEXLOWER.decode(trimmed_address.as_bytes()).map_err(|e|
        StdError::generic_err(format!("Failed to decode address: {}", e))
    )?;

    if decoded.len() != 20 {
        return Err(StdError::generic_err("Invalid address length"));
    }

    let mut padded = vec![0u8; 12]; //12 bytes of padding to reach 32 bytes total
    padded.extend_from_slice(&decoded);

    Ok(padded)
}

pub fn validate_payments(deps:&DepsMut<EvmQueryWrapper>, groups: &Vec<MintGroup>) -> StdResult<()> {
    for group in groups.iter() {
        for payment in group.payments.iter() {
            match payment.payment_type {
                PaymentType::Native => {
                    if payment.amount.is_none() || payment.amount.unwrap().is_zero() {
                        return Err(StdError::generic_err("Native payment amount is required"));
                    }
                    if payment.amount.unwrap().is_zero() {
                        return Err(
                            StdError::generic_err(
                                "Native payment amount cannot be zero. If you want it to be free, remove the payment."
                            )
                        );
                    }
                    //args[0] must be the recipient address
                    if payment.args.len() < 1 {
                        return Err(StdError::generic_err("Native payment recipient is required"));
                    }
                    if let Err(_) = deps.api.addr_validate(&payment.args[0]) {
                        return Err(StdError::generic_err("Invalid recipient address"));
                    }
                }
                PaymentType::Cw20 => {
                    if payment.amount.is_none() {
                        return Err(StdError::generic_err("Cw20 payment amount is required"));
                    }
                    //args[0] must be the recipient address and args[1] must be the token contract address
                    if payment.args.len() < 2 {
                        return Err(
                            StdError::generic_err(
                                "Cw20 payment recipient and token contract address are required"
                            )
                        );
                    }
                    if let Err(_) =  deps.api.addr_validate(&payment.args[0]) {
                        return Err(StdError::generic_err("Invalid recipient address"));
                    }
                    if let Err(_) =  deps.api.addr_validate(&payment.args[1]) {
                        return Err(StdError::generic_err("Invalid token contract address"));
                    }
                }
                PaymentType::Cw20Burn => {
                    if payment.amount.is_none() {
                        return Err(StdError::generic_err("Cw20 burn amount is required"));
                    }
                    //args[0] must be the recipient address and args[1] must be the token contract address
                    if payment.args.len() < 1 {
                        return Err(
                            StdError::generic_err(
                                "Cw20 burn token contract address is required"
                            )
                        );
                    }
                    if let Err(_) =  deps.api.addr_validate(&payment.args[0]) {
                        return Err(StdError::generic_err("Invalid token contract address"));
                    }
                }
                _ => {
                    return Err(StdError::generic_err("Invalid payment type"));
                }
            }
        }
    }

    Ok(())
}

pub fn validate_groups(collection_type: &String, groups: &Vec<MintGroup>) -> StdResult<()> {
    if collection_type == "404" {
        for group in groups.iter() {
            if group.batch_size.is_none() || group.batch_size.unwrap().is_zero() {
                return Err(StdError::generic_err("Batch size is required for 404 collection"));
            }
        }
    }

    Ok(())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join("")
}

/// Converts a Bech32 address to an Ethereum address.
pub fn convert_bech32_to_hex(bech32_address: &str) -> StdResult<String> {
 
    let (_hrp, data, _variant) = decode(bech32_address).map_err(|e| StdError::generic_err(format!("Bech32 decode error: {:?}", e)))?;
    let bytes = Vec::<u8>::from_base32(&data).map_err(|e| StdError::generic_err(format!("Base32 decode error: {:?}", e)))?;

    // Extract the last 20 bytes (Ethereum address)
    if bytes.len() < 20 {
        return Err(StdError::generic_err("Invalid Bech32 address length"));
    }
    let eth_address_bytes = &bytes[bytes.len() - 20..];

    let eth_address_hex = bytes_to_hex(eth_address_bytes);

    Ok(format!("0x{}", eth_address_hex))
}