use k256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};

/// Verifies an ECDSA signature against a pre-hashed message (32-byte hash).
///
/// This function expects the message to already be hashed (e.g., SHA256).
/// It does NOT hash the message again.
///
/// # Arguments
/// * `pubkey` - The public key in SEC1 format (33 bytes compressed or 65 bytes uncompressed)
/// * `sig` - The signature as r||s (64 bytes)
/// * `message_hash` - The pre-computed hash of the message (must be exactly 32 bytes)
pub fn verify_signature_hash(
    pubkey: &[u8],
    sig: &[u8],
    message_hash: &[u8],
) -> Result<(), &'static str> {
    if message_hash.len() != 32 {
        return Err("Message hash must be 32 bytes");
    }

    let verifying_key = VerifyingKey::from_sec1_bytes(pubkey)
        .map_err(|_| "Invalid public key")?;

    let signature = Signature::from_slice(sig)
        .map_err(|_| "Invalid signature")?;

    verifying_key
        .verify_prehash(message_hash, &signature)
        .map_err(|_| "Signature verification failed")
}