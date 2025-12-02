use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use base64::{engine::general_purpose, Engine as _};

// 为了 MVP，使用一个硬编码的 Key (实际生产中应使用 OS Keyring)
const MASTER_KEY_BYTES: [u8; 32] = [
    0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF,
    0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF,
    0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF,
    0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF,
];

pub fn encrypt_api_key(api_key: &str) -> Result<String, String> {
    let unbound_key = UnboundKey::new(&AES_256_GCM, &MASTER_KEY_BYTES)
        .map_err(|_| "Failed to create key".to_string())?;
    let key = LessSafeKey::new(unbound_key);

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes).map_err(|_| "Failed to generate nonce".to_string())?;
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|_| "Failed to create nonce".to_string())?;

    let mut in_out = api_key.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Encryption failed".to_string())?;

    // Prepend nonce to the output
    let mut result = nonce_bytes.to_vec();
    result.append(&mut in_out);

    Ok(general_purpose::STANDARD.encode(result))
}

pub fn decrypt_api_key(encrypted_base64: &str) -> Result<String, String> {
    let data = general_purpose::STANDARD
        .decode(encrypted_base64)
        .map_err(|_| "Base64 decode failed".to_string())?;

    if data.len() < 12 {
        return Err("Invalid data length".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let mut in_out = ciphertext.to_vec();

    let unbound_key = UnboundKey::new(&AES_256_GCM, &MASTER_KEY_BYTES)
        .map_err(|_| "Failed to create key".to_string())?;
    let key = LessSafeKey::new(unbound_key);
    
    let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| "Failed to create nonce".to_string())?;

    let decrypted_data = key.open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Decryption failed".to_string())?;

    String::from_utf8(decrypted_data.to_vec()).map_err(|_| "Invalid UTF-8".to_string())
}