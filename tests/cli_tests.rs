use assert_cmd::Command;
use std::{env, fs::File, io::prelude::*};
use tempfile::tempdir;

#[test]
fn dotenv_vault_cli() {
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

    {
        // Run the CLI program with dotenv-vault run -- <program> <arguments>
        let mut cmd = Command::cargo_bin("dotenv-vault").unwrap();
        if cfg!(windows) {
            cmd.args(["run", "--", "cmd", "/C", "echo %ALPHA%"]);
        } else {
            cmd.args(["run", "--", "bash", "-c", "printenv ALPHA"]);
        }

        cmd.assert().success();
        let output = cmd.output().unwrap();
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "zeta\n");
    }

    env::set_current_dir(&cwd).unwrap();

    {
        env::set_var("ALPHA", "beta");

        // override the existing environment variables and specify the current working directory
        let mut cmd = Command::cargo_bin("dotenv-vault").unwrap();
        if cfg!(windows) {
            cmd.args([
                "run",
                "--cwd",
                tmp.path().to_string_lossy().as_ref(),
                "--override",
                "--",
                "cmd",
                "/C",
                "echo %ALPHA%",
            ]);
        } else {
            cmd.args([
                "run",
                "--cwd",
                tmp.path().to_string_lossy().as_ref(),
                "--override",
                "--",
                "bash",
                "-c",
                "printenv ALPHA",
            ]);
        }

        cmd.assert().success();
        let output = cmd.output().unwrap();
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "zeta\n");
    }

    tmp.close().unwrap();
    env::remove_var("DOTENV_KEY");
    env::set_current_dir(cwd).unwrap();
}
