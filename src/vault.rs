use super::errors::{Error, Result};
use super::log::{info, warn};

use std::{env, path::PathBuf};

/// Vault data
pub struct Vault {
    /// Dotenv key
    key: Option<String>,

    /// Vault path
    path: Option<PathBuf>,
}

impl Vault {
    /// Create a new Vault using the *DOTENV_KEY* environment variable and a *.env.vault* file in
    /// the current directory
    pub fn new() -> Self {
        let key = env::var("DOTENV_KEY").map_or(None, |key| Some(key.trim().to_string()));
        let path = env::current_dir().map_or(None, |path| Some(path.join(".env.vault")));

        Self { key, path }
    }

    /// Load the *.env.vault* file into the environment, or load a regular *.env* file if a *.env.vault* file
    /// cannot be found and parsed
    pub fn load(&self) -> Result<()> {
        match self.find()? {
            Some(vault) => {
                dotenvy::from_read(&vault[..])?;
            }
            None => {
                dotenvy::dotenv()?;
            }
        }

        Ok(())
    }

    /// Load the .env.vault file into the environment, or load a regular *.env* file if a .env.vault file
    /// cannot be found and parsed, overriding any existing values in the environment
    pub fn load_override(&self) -> Result<()> {
        match &self.find()? {
            Some(vault) => {
                dotenvy::from_read_override(&vault[..])?;
            }
            None => {
                dotenvy::dotenv_override()?;
            }
        }

        Ok(())
    }

    /// Find and parse a *.env.vault* file
    ///
    /// # Returns
    /// A result containing an `Option<Vec<u8>>` for the decrypted vault contents.
    ///
    /// If the dotenv key or vault file is missing it returns None, indicating a fallback to a
    /// regular .env file.
    fn find(&self) -> Result<Option<Vec<u8>>> {
        if self.key.is_none() {
            if !cfg!(debug_assertions) {
                warn("You are using dotenv-vault in a production environment, but you haven't set DOTENV_KEY. Did you forget? Run 'npx dotenv-vault keys' to view your DOTENV_KEY.");
            }
            return Ok(None);
        }

        if self.path.as_ref().map_or(false, |path| path.exists()) {
            info("Loading env from encrypted .env.vault");
            let vault = self.parse()?;
            return Ok(Some(vault));
        }

        warn("You set a DOTENV_KEY but you are missing a .env.vault file. Did you forget to build it? Run 'npx dotenv-vault build'.");
        Ok(None)
    }

    /// Decrypt the contents of the *.env.vault* file using AES-256-GCM
    ///
    /// # Arguments
    /// - `encrypted` - The encrypted vault string
    /// - `key` - The decryption key
    fn decrypt(&self, encrypted: String, key: String) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{consts::U12, Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };
        use base64::{engine::general_purpose, Engine as _};

        let key_len = key.len();
        if key_len < 64 {
            return Err(Error::InvalidKey);
        }
        let key = key[key.len() - 64..].to_string();
        let key = hex::decode(key)?;
        let ciphertext = general_purpose::STANDARD.decode(encrypted)?;

        let nonce = &ciphertext[0..12];
        let ciphertext = &ciphertext[12..];

        let key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::<U12>::from_slice(nonce);

        let plaintext = cipher.decrypt(nonce, ciphertext)?;

        Ok(plaintext)
    }

    /// Parse the dotenv key uri into a key and environment
    ///
    /// # Arguments
    /// - `dotenv_key` - The dotenv key uri
    ///
    /// # Returns
    /// A `Result` containing a tuple of `(key, environment)`
    fn instructions(&self, dotenv_key: &str) -> Result<(String, String)> {
        let url = url::Url::parse(dotenv_key)?;

        if url.scheme() != "dotenv" {
            return Err(Error::InvalidScheme);
        }

        let key = match url.password() {
            Some(key) => key.to_string(),
            None => return Err(Error::MissingKey),
        };

        let environment = match url.query_pairs().find(|(k, _)| k == "environment") {
            Some((_, environment)) => environment.to_string(),
            None => return Err(Error::MissingEnvironment),
        };

        let environment_key = format!("DOTENV_VAULT_{}", environment.to_uppercase());
        Ok((key, environment_key))
    }

    /// Parse the *.env.vault* file into a `Vec<u8>`
    ///
    /// # Returns
    /// A `Result` containing a `Vec<u8>` of the decrypted vault contents
    fn parse(&self) -> Result<Vec<u8>> {
        let keys = match self.key.as_ref() {
            Some(key) => key,
            None => return Err(Error::KeyNotFound),
        };

        let path = match self.path.as_ref() {
            Some(path) => path,
            None => return Err(Error::VaultNotFound),
        };

        for key in keys.split(',') {
            if let Ok(decrypted) = self
                .instructions(key)
                .and_then(|(k, e)| {
                    let vault = dotenvy::from_path_iter(path)?;
                    let maybe_ciphertext = vault.into_iter().find(|item| match item {
                        Ok((k, _)) => k == &e,
                        _ => false,
                    });
                    let ciphertext = match maybe_ciphertext {
                        Some(Ok((_, c))) => c,
                        _ => return Err(Error::EnvironmentNotFound(e)),
                    };

                    Ok((ciphertext, k))
                })
                .and_then(|(c, k)| self.decrypt(c, k))
            {
                return Ok(decrypted);
            }
        }

        Err(Error::InvalidKey)
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::{fs::File, io::prelude::*};

    use super::*;

    #[test]
    #[serial] // Run serially due to env modifications
    fn new_ok() {
        std::env::set_var("DOTENV_KEY", "dotenv://:testkey");
        let vault = Vault::new();
        assert!(vault.key.is_some());
        assert!(vault.key.unwrap() == "dotenv://:testkey");
        assert!(vault.path.is_some());
        assert!(vault.path.unwrap() == env::current_dir().unwrap().join(".env.vault"));
        std::env::remove_var("DOTENV_KEY");
    }

    #[test]
    fn instructions_ok() {
        let vault = Vault::new();
        let instructions = vault
            .instructions("dotenv://:key_1234@dotenv.org/vault/.env.vault?environment=production");

        assert!(instructions.is_ok());
        let (key, environment) = instructions.unwrap();
        assert_eq!(key, "key_1234");
        assert_eq!(environment, "DOTENV_VAULT_PRODUCTION");
    }

    #[test]
    fn instructions_invalid_scheme() {
        let vault = Vault::new();
        let instructions =
            vault.instructions("invalid://dotenv.org/vault/.env.vault?environment=production");

        assert!(instructions.is_err());
        assert!(matches!(instructions.unwrap_err(), Error::InvalidScheme));
    }

    #[test]
    fn instructions_missing_key() {
        let vault = Vault::new();
        let instructions =
            vault.instructions("dotenv://dotenv.org/vault/.env.vault?environment=production");

        assert!(instructions.is_err());
        assert!(matches!(instructions.unwrap_err(), Error::MissingKey));
    }

    #[test]
    fn instructions_missing_environment() {
        let vault = Vault::new();
        let instructions = vault.instructions("dotenv://:key_1234@dotenv.org/vault/.env.vault");

        assert!(instructions.is_err());
        assert!(matches!(
            instructions.unwrap_err(),
            Error::MissingEnvironment
        ));
    }

    #[test]
    fn decrypt_ok() {
        let vault = Vault::new();
        let decrypted = vault.decrypt(
            "s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R".into(),
            "ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00".into(),
        );
        assert!(decrypted.is_ok());
        assert_eq!(
            decrypted.unwrap(),
            "# development@v6\nALPHA=\"zeta\"".as_bytes()
        );
    }

    #[test]
    fn decrypt_invalid_key() {
        let vault = Vault::new();
        let decrypted = vault.decrypt(
            "s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R".into(),
            "01b08fe1173b781cce5fd1a18178c5cacdf3bb0845a8aa1b8089ac0751f7ed9c".into(),
        );
        assert!(matches!(decrypted, Err(Error::DecryptError(_))));
    }

    #[test]
    fn decrypt_invalid_ciphertext() {
        let vault = Vault::new();
        let decrypted = vault.decrypt(
            "bQ4c611kJ7kVoUNzHXEbV+bTYc/4UVeyKXXgUpyaaIiUrzOrCauLix6lxrBm4FrCql6kxBA7f/oVO5U+kLMzHA==".into(),
            "ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00".into(),
        );
        assert!(matches!(decrypted, Err(Error::DecryptError(_))));
    }

    #[test]
    fn decrypt_short_key() {
        let vault = Vault::new();
        let decrypted = vault.decrypt(
            "s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R".into(),
            "caa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00".into(),
        );
        assert!(matches!(decrypted, Err(Error::InvalidKey)));
    }

    #[test]
    fn decrypt_invalid_hex() {
        let vault = Vault::new();
        let decrypted = vault.decrypt(
            "s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R".into(),
            "XXcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00".into(),
        );
        assert!(matches!(decrypted, Err(Error::HexError(_))));
    }

    #[test]
    fn decrypt_invalid_base64() {
        let vault = Vault::new();
        let decrypted = vault.decrypt(
            "FFFFFFFs7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R"
                .into(),
            "ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00".into(),
        );
        assert!(matches!(decrypted, Err(Error::DecodeError(_))));
    }

    #[test]
    fn parse_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join(".env.vault");
        let mut vault = File::create(&vault_path).unwrap();
        vault
            .write_all("DOTENV_VAULT_DEVELOPMENT=\"s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R\"".as_bytes())
            .unwrap();
        vault.sync_all().unwrap();

        let vault = Vault {
            key: Some("dotenv://:key_ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=development".into()),
            path: Some(vault_path)
        };
        let parsed = vault.parse();

        assert!(parsed.is_ok());
        assert_eq!(
            parsed.unwrap(),
            vec![
                35, 32, 100, 101, 118, 101, 108, 111, 112, 109, 101, 110, 116, 64, 118, 54, 10, 65,
                76, 80, 72, 65, 61, 34, 122, 101, 116, 97, 34
            ]
        );

        tmp.close().unwrap();
    }

    #[test]
    fn parse_invalid_environment() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join(".env.vault");
        let mut vault = File::create(&vault_path).unwrap();
        vault
            .write_all("DOTENV_VAULT_PRODUCTION=\"s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R\"".as_bytes())
            .unwrap();
        vault.sync_all().unwrap();

        let vault = Vault {
            key: Some("dotenv://:key_ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=development".into()),
            path: Some(vault_path)
        };
        let parsed = vault.parse();

        assert!(parsed.is_err());
        assert!(matches!(parsed.unwrap_err(), Error::InvalidKey));

        tmp.close().unwrap();
    }

    #[test]
    fn parse_invalid_key() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join(".env.vault");
        let mut vault = File::create(&vault_path).unwrap();
        vault
            .write_all("DOTENV_VAULT_PRODUCTION=\"XXNYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R\"".as_bytes())
            .unwrap();
        vault.sync_all().unwrap();

        let vault = Vault {
            key: Some("dotenv://:key_ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=development".into()),
            path: Some(vault_path)
        };
        let parsed = vault.parse();

        assert!(parsed.is_err());
        assert!(matches!(parsed.unwrap_err(), Error::InvalidKey));

        tmp.close().unwrap();
    }

    #[test]
    fn parse_multiple_keys() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join(".env.vault");
        let mut vault = File::create(&vault_path).unwrap();
        vault
            .write_all("DOTENV_VAULT_PRODUCTION=\"s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R\"".as_bytes())
            .unwrap();
        vault.sync_all().unwrap();

        let vault = Vault {
            key: Some("dotenv://:key_XXcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=development,dotenv://:key_ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=production".into()),
            path: Some(vault_path)
        };
        let parsed = vault.parse();

        assert!(parsed.is_ok());
        assert_eq!(
            parsed.unwrap(),
            vec![
                35, 32, 100, 101, 118, 101, 108, 111, 112, 109, 101, 110, 116, 64, 118, 54, 10, 65,
                76, 80, 72, 65, 61, 34, 122, 101, 116, 97, 34
            ]
        );

        tmp.close().unwrap();
    }

    #[test]
    fn parse_multiple_invalid_keys() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join(".env.vault");
        let mut vault = File::create(&vault_path).unwrap();
        vault
            .write_all("DOTENV_VAULT_PRODUCTION=\"s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R\"".as_bytes())
            .unwrap();
        vault.sync_all().unwrap();

        let vault = Vault {
            key: Some("dotenv://:key_XXcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=development,dotenv://:key_XXYY6504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=production".into()),
            path: Some(vault_path)
        };
        let parsed = vault.parse();

        assert!(parsed.is_err());
        assert!(matches!(parsed.unwrap_err(), Error::InvalidKey));

        tmp.close().unwrap();
    }
}
