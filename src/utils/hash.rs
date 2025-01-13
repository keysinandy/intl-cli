use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
type HmacSha256 = Hmac<Sha256>;

pub fn sha256(message: &[u8], secret: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}

pub fn get_hash(message: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hex::encode(hasher.finalize())
}
