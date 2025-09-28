#[cfg(test)]
mod tests {
    use program::ecdsa::verify_signature_hash;

    // Include the shared test data
    include!("../../examples/tests/common/ecdsa_test_data.rs");

    #[test]
    fn test_verify_signature_hash_with_real_data() {
        println!("=== Testing ECDSA verification with real data ===");
        println!("Message: \"Hello, AVM!\"");

        // Use test data from shared ecdsa_test_data
        let pubkey = ECDSA_PUBKEY;
        let signature = ECDSA_SIGNATURE;
        let message_hash = ECDSA_MESSAGE_HASH;

        println!("Public key (uncompressed): {} bytes", pubkey.len());
        println!("Signature (r||s): {} bytes", signature.len());
        println!("Message hash (SHA256): {} bytes", message_hash.len());

        // This should verify successfully
        println!("Verifying signature...");
        let result = verify_signature_hash(pubkey, signature, message_hash);

        match &result {
            Ok(()) => println!("✓ Signature verification SUCCESSFUL"),
            Err(e) => println!("✗ Signature verification FAILED: {}", e),
        }

        assert!(result.is_ok(), "Valid signature should verify successfully");
    }

    #[test]
    fn test_verify_signature_hash_with_second_transaction_data() {
        println!("=== Testing ECDSA verification with second set of data ===");
        println!("Message: \"Test message\"");

        // Use test data from shared ecdsa_test_data
        let pubkey = ECDSA_PUBKEY_2;
        let signature = ECDSA_SIGNATURE_2;
        let message_hash = ECDSA_MESSAGE_HASH_2;

        // This should verify successfully
        println!("Verifying signature...");
        let result = verify_signature_hash(pubkey, signature, message_hash);

        match &result {
            Ok(()) => println!("✓ Signature verification SUCCESSFUL"),
            Err(e) => println!("✗ Signature verification FAILED: {}", e),
        }

        assert!(result.is_ok(), "Valid signature should verify successfully");
    }

    #[test]
    fn test_verify_signature_hash_with_invalid_signature() {
        println!("=== Testing ECDSA verification with invalid signature ===");
        // Use valid pubkey and message hash but tamper with signature

        let pubkey = [
            0x04, // uncompressed prefix
            // x coordinate (32 bytes)
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac,
            0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
            0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9,
            0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
            // y coordinate (32 bytes)
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65,
            0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8,
            0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19,
            0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8
        ];

        // Invalid signature - modified first byte
        let mut signature = ECDSA_SIGNATURE.clone();
        signature[0] = 0x00; // Modify first byte

        let message_hash = [
            0x2c, 0xf2, 0x4d, 0xba, 0x5f, 0xb0, 0xa3, 0x0e,
            0x26, 0xe8, 0x3b, 0x2a, 0xc5, 0xb9, 0xe2, 0x9e,
            0x1b, 0x16, 0x1e, 0x5c, 0x1f, 0xa7, 0x42, 0x5e,
            0x73, 0x04, 0x33, 0x62, 0x93, 0x8b, 0x98, 0x24
        ];

        // This should fail verification
        println!("Verifying tampered signature (first byte changed)...");
        let result = verify_signature_hash(&pubkey, &signature, &message_hash);

        match &result {
            Ok(()) => println!("✗ Unexpected: Invalid signature verification PASSED"),
            Err(e) => println!("✓ Expected failure: {}", e),
        }

        assert!(result.is_err(), "Invalid signature should fail verification");
    }

    #[test]
    fn test_verify_signature_hash_with_wrong_message_hash() {
        println!("=== Testing ECDSA verification with wrong message hash ===");
        // Use valid pubkey and signature but different message hash

        let pubkey = [
            0x04, // uncompressed prefix
            // x coordinate (32 bytes)
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac,
            0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
            0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9,
            0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
            // y coordinate (32 bytes)
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65,
            0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8,
            0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19,
            0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8
        ];

        let signature = [
            // r component
            0xc6, 0x04, 0x7f, 0x94, 0x41, 0xed, 0x7d, 0x6d,
            0x30, 0x45, 0x40, 0x6e, 0x95, 0xc0, 0x7c, 0xd8,
            0x5c, 0x77, 0x8e, 0x4b, 0x8c, 0xef, 0x3c, 0xa7,
            0xab, 0xac, 0x09, 0xb9, 0x5c, 0x70, 0x9e, 0xe5,
            // s component
            0x1a, 0xe1, 0x68, 0xa8, 0xc0, 0x59, 0x1b, 0xd5,
            0xc5, 0xea, 0x69, 0x11, 0xc5, 0x15, 0x90, 0x11,
            0xe3, 0x89, 0xf6, 0x8c, 0xd5, 0x63, 0x5f, 0xac,
            0xac, 0xb1, 0x35, 0xc0, 0x41, 0x0e, 0x38, 0xac
        ];

        // Wrong message hash - all zeros
        let message_hash = [0u8; 32];

        // This should fail verification
        println!("Verifying with wrong message hash (all zeros)...");
        let result = verify_signature_hash(&pubkey, &signature, &message_hash);

        match &result {
            Ok(()) => println!("✗ Unexpected: Wrong message hash verification PASSED"),
            Err(e) => println!("✓ Expected failure: {}", e),
        }

        assert!(result.is_err(), "Wrong message hash should fail verification");
    }

    #[test]
    fn test_verify_signature_hash_with_invalid_pubkey() {
        println!("=== Testing ECDSA verification with invalid public key ===");
        // Invalid public key format
        let invalid_pubkey = [0xFF; 65]; // Invalid pubkey

        let signature = [
            // r component
            0xc6, 0x04, 0x7f, 0x94, 0x41, 0xed, 0x7d, 0x6d,
            0x30, 0x45, 0x40, 0x6e, 0x95, 0xc0, 0x7c, 0xd8,
            0x5c, 0x77, 0x8e, 0x4b, 0x8c, 0xef, 0x3c, 0xa7,
            0xab, 0xac, 0x09, 0xb9, 0x5c, 0x70, 0x9e, 0xe5,
            // s component
            0x1a, 0xe1, 0x68, 0xa8, 0xc0, 0x59, 0x1b, 0xd5,
            0xc5, 0xea, 0x69, 0x11, 0xc5, 0x15, 0x90, 0x11,
            0xe3, 0x89, 0xf6, 0x8c, 0xd5, 0x63, 0x5f, 0xac,
            0xac, 0xb1, 0x35, 0xc0, 0x41, 0x0e, 0x38, 0xac
        ];

        let message_hash = [
            0x2c, 0xf2, 0x4d, 0xba, 0x5f, 0xb0, 0xa3, 0x0e,
            0x26, 0xe8, 0x3b, 0x2a, 0xc5, 0xb9, 0xe2, 0x9e,
            0x1b, 0x16, 0x1e, 0x5c, 0x1f, 0xa7, 0x42, 0x5e,
            0x73, 0x04, 0x33, 0x62, 0x93, 0x8b, 0x98, 0x24
        ];

        println!("Verifying with invalid public key (all 0xFF bytes)...");
        let result = verify_signature_hash(&invalid_pubkey, &signature, &message_hash);

        match &result {
            Ok(()) => println!("✗ Unexpected: Invalid pubkey verification PASSED"),
            Err(e) => println!("✓ Expected failure: {}", e),
        }

        assert!(result.is_err(), "Invalid public key should return error");
        assert_eq!(result.unwrap_err(), "Invalid public key");
    }

    #[test]
    fn test_verify_signature_hash_with_wrong_hash_size() {
        println!("=== Testing ECDSA verification with wrong hash size ===");
        let pubkey = [
            0x04, // uncompressed prefix
            // x coordinate (32 bytes)
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac,
            0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
            0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9,
            0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
            // y coordinate (32 bytes)
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65,
            0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8,
            0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19,
            0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8
        ];

        let signature = [0u8; 64]; // dummy signature

        // Wrong size message hash (not 32 bytes)
        let wrong_size_hash = [0u8; 16];

        println!("Verifying with wrong hash size (16 bytes instead of 32)...");
        let result = verify_signature_hash(&pubkey, &signature, &wrong_size_hash);

        match &result {
            Ok(()) => println!("✗ Unexpected: Wrong hash size verification PASSED"),
            Err(e) => println!("✓ Expected failure: {}", e),
        }

        assert!(result.is_err(), "Wrong hash size should return error");
        assert_eq!(result.unwrap_err(), "Message hash must be 32 bytes");
    }
}