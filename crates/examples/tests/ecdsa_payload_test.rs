use k256::ecdsa::{Signature, SigningKey, VerifyingKey, signature::hazmat::PrehashVerifier};

#[path = "common/ecdsa.rs"]
mod ecdsa;

#[test]
fn test_ecdsa_payload_is_valid() {
    // Build payload using the shared helper.
    let payload = ecdsa::build_ecdsa_payload();

    let pk_len = payload[0] as usize;
    let pubkey = &payload[1..1 + pk_len];
    let sig = &payload[1 + pk_len..1 + pk_len + 64];
    let hash = &payload[1 + pk_len + 64..];

    // Ensure the public key matches the hardcoded secret key we use for signing.
    let signing_key =
        SigningKey::from_bytes(&ecdsa::ECDSA_SK_BYTES.into()).expect("valid sk bytes");
    let expected_vk = signing_key.verifying_key();
    let payload_vk = VerifyingKey::from_sec1_bytes(pubkey).expect("valid pubkey in payload");
    assert_eq!(
        expected_vk.to_encoded_point(true),
        payload_vk.to_encoded_point(true),
        "payload pubkey should match the signing key"
    );

    // Verify the signature against the known hash.
    let sig = Signature::from_slice(sig).expect("valid signature bytes");
    payload_vk
        .verify_prehash(hash, &sig)
        .expect("signature should verify for payload hash");

    // Double-check we used the expected hash constant.
    assert_eq!(
        hash,
        &ecdsa::ECDSA_HASH,
        "payload hash should match the test constant"
    );
}
