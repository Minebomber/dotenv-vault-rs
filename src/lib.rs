//! Extends the [dotenvy](https://crates.io/crates/dotenvy) crate with *.env.vault* file support.
//! The extended standard lets you load encrypted secrets from your *.env.vault* file in production (and other) environments.

mod errors;
mod log;
mod vault;

pub use dotenvy;
pub use errors::Error;

use errors::Result;
use vault::Vault;

/// Loads the *.env.vault* file from [`env::current_dir`](std::env::current_dir) using the *DOTENV_KEY* environment
/// variable.
///
/// If the key or vault cannot be found, a regular *.env* file is loaded instead.
///
/// If variables with the same names already exist in the environment, then their values will be
/// preserved.
///
/// Where multiple declarations for the same environment variable exist in your *.env*
/// file, the *first one* is applied.
///
/// If you wish to ensure all variables are loaded from your *.env.vault* file, ignoring variables
/// already existing in the environment, then use [`dotenv_override`] instead.
///
/// An error will be returned if the file is not found.
///
/// # Examples
/// ```no_run
/// fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     dotenv_vault::dotenv()?;
///     Ok(())
/// }
/// ```
pub fn dotenv() -> Result<()> {
    Vault::new().load()
}

/// Loads all variables into the environment, overriding any existing environment variables of the
/// same name.
///
/// If the key or vault cannot be found, a regular *.env* file is loaded instead.
///
/// Where multiple declarations for the same environment variable exist in your *.env* file, the
/// *last one* is applied.
///
/// If you want the existing environment to take precedence,
/// or if you want to be able to override environment variables on the command line,
/// then use [`dotenv`] instead.
///
/// # Examples
/// ```no_run
/// fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     dotenv_vault::dotenv_override()?;
///     Ok(())
/// }
/// ```
pub fn dotenv_override() -> Result<()> {
    Vault::new().load_override()
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::{env, fs::File, io::prelude::*};
    use tempfile::tempdir;

    #[test]
    #[serial] // Run serially due to env modifications
    fn dotenv_ok() {
        env::set_var("DOTENV_KEY", "dotenv://:key_ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=production");

        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join(".env.vault");
        let mut vault = File::create(&vault_path).unwrap();
        vault
            .write_all("DOTENV_VAULT_PRODUCTION=\"s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R\"".as_bytes())
            .unwrap();
        vault.sync_all().unwrap();

        let cwd = env::current_dir().unwrap();
        env::set_current_dir(&tmp).unwrap();

        let result = super::dotenv();
        assert!(result.is_ok());

        let from_vault = env::var("ALPHA");
        assert!(from_vault.is_ok());
        assert!(from_vault.unwrap() == "zeta");

        tmp.close().unwrap();
        env::remove_var("DOTENV_KEY");
        env::remove_var("ALPHA");
        env::set_current_dir(cwd).unwrap();
    }

    #[test]
    #[serial] // Run serially due to env modifications
    fn dotenv_fallback_to_env() {
        let tmp = tempdir().unwrap();
        let env_path = tmp.path().join(".env");
        let mut env = File::create(&env_path).unwrap();
        env.write_all("TESTKEY=\"from .env\"".as_bytes()).unwrap();
        env.sync_all().unwrap();

        let cwd = env::current_dir().unwrap();
        env::set_current_dir(&tmp).unwrap();

        let result = super::dotenv();
        assert!(result.is_ok());

        let from_env = env::var("TESTKEY");
        assert!(from_env.is_ok());
        assert!(from_env.unwrap() == "from .env");

        tmp.close().unwrap();
        env::remove_var("TESTKEY");
        env::set_current_dir(cwd).unwrap();
    }

    #[test]
    #[serial] // Run serially due to env modifications
    fn dotenv_override_ok() {
        env::set_var("DOTENV_KEY", "dotenv://:key_ddcaa26504cd70a6fef9801901c3981538563a1767c297cb8416e8a38c62fe00@dotenv.local/vault/.env.vault?environment=production");

        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join(".env.vault");
        let mut vault = File::create(&vault_path).unwrap();
        vault
            .write_all("DOTENV_VAULT_PRODUCTION=\"s7NYXa809k/bVSPwIAmJhPJmEGTtU0hG58hOZy7I0ix6y5HP8LsHBsZCYC/gw5DDFy5DgOcyd18R\"".as_bytes())
            .unwrap();
        vault.sync_all().unwrap();

        let cwd = env::current_dir().unwrap();
        env::set_current_dir(&tmp).unwrap();

        env::set_var("ALPHA", "beta");

        let result = super::dotenv_override();
        assert!(result.is_ok());

        let from_vault = env::var("ALPHA");
        assert!(from_vault.is_ok());
        assert!(from_vault.unwrap() == "zeta");

        tmp.close().unwrap();
        env::remove_var("DOTENV_KEY");
        env::remove_var("ALPHA");
        env::set_current_dir(cwd).unwrap();
    }

    #[test]
    #[serial] // Run serially due to env modifications
    fn dotenv_override_fallback_to_env() {
        let tmp = tempdir().unwrap();
        let env_path = tmp.path().join(".env");
        let mut env = File::create(&env_path).unwrap();
        env.write_all("TESTKEY=\"from .env\"".as_bytes()).unwrap();
        env.sync_all().unwrap();

        let cwd = env::current_dir().unwrap();
        env::set_current_dir(&tmp).unwrap();

        env::set_var("TESTKEY", "helloworld");

        let result = super::dotenv_override();
        assert!(result.is_ok());

        let from_env = env::var("TESTKEY");
        assert!(from_env.is_ok());
        assert!(from_env.unwrap() == "from .env");

        tmp.close().unwrap();
        env::remove_var("TESTKEY");
        env::set_current_dir(cwd).unwrap();
    }
}
