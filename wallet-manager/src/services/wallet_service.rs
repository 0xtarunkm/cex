use anyhow::Result;
use bip39::Mnemonic;
use ed25519_hd_key::derive_from_path;
use std::env;

pub struct WalletService {
    solana_wallet_count: usize,
}

impl WalletService {
    pub fn new() -> Self {
        Self {
            solana_wallet_count: 0,
        }
    }

    pub fn generate_wallet(&mut self) -> Result<(String, String)> {
        let phrase = env::var("MNEMONIC").expect("MNEMONIC must be set in .env file");
        let mnemonic = Mnemonic::parse(&phrase).expect("Invalid mnemonic phrase");
        let seed = Mnemonic::to_seed(&mnemonic, "");

        let path = format!("m/44'/501'/{}'/0'", self.solana_wallet_count);
        self.solana_wallet_count += 1;
        let derivation_seed = derive_from_path(&path, &seed).0;

        let seed_bytes: [u8; 32] = derivation_seed[..32].try_into().unwrap();
        let mut public_key = [0u8; 32];
        let mut secret_key = [0u8; 64];

        sodalite::sign_keypair_seed(&mut public_key, &mut secret_key, &seed_bytes);

        Ok((
            bs58::encode(&public_key).into_string(),
            bs58::encode(&secret_key).into_string(),
        ))
    }
}
