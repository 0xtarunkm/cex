use anyhow::Result;
use bip39::Mnemonic;
use ed25519_hd_key::derive_from_path;
use hex;
use secp256k1::{Secp256k1, SecretKey};
use std::env;
use tiny_keccak::{Hasher, Keccak};

pub struct WalletService {
    solana_wallet_count: usize,
    ethereum_wallet_count: usize,
}

impl WalletService {
    pub fn new() -> Self {
        Self {
            solana_wallet_count: 0,
            ethereum_wallet_count: 0,
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

    pub fn generate_ethereum_wallet(&mut self) -> Result<(String, String)> {
        let phrase = env::var("MNEMONIC").expect("MNEMONIC must be set in .env file");
        let mnemonic = Mnemonic::parse(&phrase).expect("Invalid mnemonic phrase");
        let seed = Mnemonic::to_seed(&mnemonic, "");

        let path = format!("m/44'/60'/0'/0/{}", self.ethereum_wallet_count);
        self.ethereum_wallet_count += 1;

        let derivation_seed = derive_from_path(&path, &seed).0;
        let private_key_bytes: [u8; 32] = derivation_seed[..32].try_into().unwrap();
        
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&private_key_bytes)?;
        
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
        
        let public_key_bytes = public_key.serialize_uncompressed()[1..].to_vec();
        
        let mut keccak = Keccak::v256();
        let mut hash = [0u8; 32];
        keccak.update(&public_key_bytes);
        keccak.finalize(&mut hash);
        
        let address_bytes = &hash[12..32];
        
        let address = format!("0x{}", hex::encode(address_bytes));
        let private_key = hex::encode(private_key_bytes);
        
        Ok((address, private_key))
    }
}
