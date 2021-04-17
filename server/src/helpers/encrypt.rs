use magic_crypt::MagicCryptTrait;

pub fn encrypt(value: &String) -> String {
    let key: String = std::env::var("MAGIC_KEY").expect("MAGIC_KEY not set");
    let mc = new_magic_crypt!(key, 256);
    let base64 = mc.encrypt_str_to_base64(value);
    base64
}

pub fn decrypt(value: &String) -> String {
    let key: String = std::env::var("MAGIC_KEY").expect("MAGIC_KEY not set");
    let mc = new_magic_crypt!(key, 256);
    let data = mc.decrypt_base64_to_string(value).unwrap();
    data
}
