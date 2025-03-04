/// Returns hash encoded in hex of the provided `data`.
pub fn hash_256<T: AsRef<[u8]>>(data: T) -> String {
    let hash = near_sdk::env::sha256_array(data.as_ref());
    hex::encode(hash)
}

#[test]
fn test_hash_256() {
    const HELLO_HASH: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    assert_eq!(hash_256(b"hello"), HELLO_HASH);
    assert_ne!(hash_256(b"hell0"), HELLO_HASH);
}
