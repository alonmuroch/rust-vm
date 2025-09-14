use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

pub fn verify_signature(
    pubkey: &[u8],
    sig: &[u8],
    message_hash: &[u8],
) -> Result<(), &'static str> {
    let verifying_key = VerifyingKey::from_sec1_bytes(pubkey)
        .map_err(|_| "Invalid public key")?;

    let signature = Signature::from_slice(sig)
        .map_err(|_| "Invalid signature")?;

    if message_hash.len() != 32 {
        return Err("Message hash must be 32 bytes");
    }

    verifying_key
        .verify(message_hash, &signature)
        .map_err(|_| "Signature verification failed")
}