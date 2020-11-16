extern crate bcrypt;

use bcrypt::{hash, verify};
const DEFAULT_COST: u32 = 8;

pub fn encrypt_password(password: &String) -> String {
    let hashed = hash(password, DEFAULT_COST).unwrap();
    hashed
}

pub fn compare_password(user_password: &String, given_password: &String) -> bool {
    let valid = verify(given_password, user_password).unwrap();
    valid
}
