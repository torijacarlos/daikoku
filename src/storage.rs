use std::path::PathBuf;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
    Nonce, // Or `Aes128Gcm`
};

use crate::{alias::DkkResult, error::DkkError, models::Wallet, ui::DkkUiState, Dkk};

pub fn load(app: &mut Dkk) {
    if let DkkUiState::Init = app.state {
        app.available_wallets = get_all_wallets_locations();
    }
}

fn left_pad(pin: &String, len: usize) -> String {
    if pin.len() < len {
        let mut pad = (0..(len - pin.len())).map(|_| " ").collect::<String>();
        pad.push_str(pin);
        return pad;
    }
    pin.clone()
}

pub fn export(wallet: &Wallet, pin: &String, key: &String) {
    let wallet_string = ron::to_string(&wallet);
    if let Ok(ws) = wallet_string {
        let cipher = Aes256Gcm::new(left_pad(key, 32).as_bytes().into());
        let pin = left_pad(pin, 12);
        let nonce = Nonce::from_slice(pin.as_bytes()); // 96-bits; unique per message
        if let Ok(ciphertext) = cipher.encrypt(nonce, ws.as_bytes()) {
            if let Ok(mut location) = get_storage_location() {
                let file_name = wallet.alias.to_lowercase().replace(" ", "_");
                location.push(file_name);
                let _ = std::fs::write(location, ciphertext);
            }
        }
    }
}

pub fn import(wallet_file: PathBuf, pin: &String, key: &String) -> DkkResult<Wallet> {
    let cipher = Aes256Gcm::new(left_pad(key, 32).as_bytes().into());
    let pin = left_pad(pin, 12);
    let nonce = Nonce::from_slice(pin.as_bytes()); // 96-bits; unique per message
    let wallet_string = std::fs::read(wallet_file)?;
    if let Ok(wallet_string) = cipher.decrypt(nonce, wallet_string.as_slice()) {
        let wallet_string = std::str::from_utf8(&wallet_string)?;
        return Ok(ron::from_str::<Wallet>(wallet_string)?);
    }
    Err(DkkError::NotAuthorized)
}

fn get_storage_location() -> DkkResult<PathBuf> {
    let mut location = home::home_dir().unwrap();
    location.push(".atelier");
    location.push("daikoku");
    let _ = std::fs::create_dir_all(&location);
    Ok(location)
}

pub fn get_all_wallets_locations() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = vec![];
    if let Ok(location) = get_storage_location() {
        if let Ok(r) = std::fs::read_dir(location) {
            for dir_entry in r {
                if let Ok(entry) = dir_entry {
                    files.push(entry.path());
                }
            }
        }
    }
    files
}
