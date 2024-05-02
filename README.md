# dotenv-vault-rs

[![crates.io](https://img.shields.io/crates/v/dotenv-vault.svg)](https://crates.io/crates/dotenv-vault)
[![msrv
1.64.0](https://img.shields.io/badge/msrv-1.64.0-dea584.svg?logo=rust)](https://github.com/rust-lang/rust/releases/tag/1.64.0)
[![ci](https://github.com/Minebomber/dotenv-vault-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Minebomber/dotenv-vault-rs/actions/workflows/ci.yml)
[![docs](https://img.shields.io/docsrs/dotenv-vault?logo=docs.rs)](https://docs.rs/dotenv-vault/)

Extends the [dotenvy](https://github.com/allan2/dotenvy) crate with `.env.vault` file support.

The extended standard lets you load encrypted secrets from your `.env.vault` file in production (and other) environments.

* [Install](#install)
* [Usage (.env)](#usage)
* [Deploying (.env.vault)](#deploying)
* [Multiple Environments](#manage-multiple-environments)
* [FAQ](#faq)
* [Changelog](./CHANGELOG.md)

## Install CLI

The dotenv-vault CLI allows loading the `.env.vault` file and run the given program with the environment variables set.

```shell
cargo install dotenv-vault --features cli
```

## Usage CLI

```shell
dotenv-vault run -- some_program arg1 arg2
```

or run at a different working directory that contains the `.env.vault` and override existing environment variables:

```shell
dotenv-vault run --cwd ./some_folder --override -- some_program arg1 arg2
```

## Install

```shell
cargo add dotenv-vault
```

## Usage

Development usage works just like [dotenvy](https://github.com/allan2/dotenvy).

Add your application configuration to your `.env` file in the root of your project:

```shell
# .env
S3_BUCKET=YOURS3BUCKET
SECRET_KEY=YOURSECRETKEYGOESHERE
```

As early as possible in your application, import and configure dotenv-vault:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv_vault::dotenv()?;

    let s3_bucket = std::env::var("S3_BUCKET")?;
    let secret_key = std::env::var("SECRET_KEY")?;

    // now do something with s3 or whatever

    Ok(())
}
```

That's it! `std::env::var` has the keys and values you defined in your `.env` file. Continue using it this way in development. It works just like [dotenvy](https://github.com/allan2/dotenvy).

## Deploying

Encrypt your environment settings by doing:

```shell
npx dotenv-vault local build
```

This will create an encrypted `.env.vault` file along with a
`.env.keys` file containing the encryption keys. Set the
`DOTENV_KEY` environment variable by copying and pasting
the key value from the `.env.keys` file onto your server
or cloud provider. For example in heroku:

```shell
heroku config:set DOTENV_KEY=<key string from .env.keys>
```

Commit your .env.vault file safely to code and deploy. Your .env.vault fill be decrypted on boot, its environment variables injected, and your app work as expected.

Note that when the `DOTENV_KEY` environment variable is set,
environment settings will *always* be loaded from the `.env.vault`
file in the project root. For development use, you can leave the
`DOTENV_KEY` environment variable unset and fall back on the
`dotenvy` behaviour of loading from `.env` or a specified set of
files (see [here in the `dotenvy`
README](https://github.com/allan2/dotenvy#usage) for the details).

## Manage Multiple Environments

You have two options for managing multiple environments - locally managed or vault managed - both use [dotenv-vault](https://github.com/dotenv-org/dotenv-vault).

Locally managed never makes a remote API call. It is completely managed on your machine. Vault managed adds conveniences like backing up your .env file, secure sharing across your team, access permissions, and version history. Choose what works best for you.

#### Locally Managed

Create a `.env.production` file in the root of your project and put your production values there.

```shell
# .env.production
S3_BUCKET="PRODUCTION_S3BUCKET"
SECRET_KEY="PRODUCTION_SECRETKEYGOESHERE"
```

Rebuild your `.env.vault` file.

```shell
npx dotenv-vault local build
```

View your `.env.keys` file. There is a production `DOTENV_KEY` that pairs with the `DOTENV_VAULT_PRODUCTION` cipher in your `.env.vault` file.

Set the production `DOTENV_KEY` on your server, recommit your `.env.vault` file to code, and deploy. That's it!

Your .env.vault fill be decrypted on boot, its production environment variables injected, and your app work as expected.

#### Vault Managed

Sync your .env file. Run the push command and follow the instructions. [learn more](https://www.dotenv.org/docs/sync/quickstart)

```
$ npx dotenv-vault push
```

Manage multiple environments with the included UI. [learn more](https://www.dotenv.org/docs/tutorials/environments)

```
$ npx dotenv-vault open
```

Build your `.env.vault` file with multiple environments.

```
$ npx dotenv-vault build
```

Access your `DOTENV_KEY`.

```
$ npx dotenv-vault keys
```

Set the production `DOTENV_KEY` on your server, recommit your `.env.vault` file to code, and deploy. That's it!

## FAQ

#### What happens if `DOTENV_KEY` is not set?

Dotenv Vault gracefully falls back to
[dotenvy](https://github.com/allan2/dotenvy) when `DOTENV_KEY` is not
set. This is the default for development so that you can focus on
editing your `.env` file and save the `build` command until you are
ready to deploy those environment variables changes.

#### Should I commit my `.env` file?

No. We **strongly** recommend against committing your `.env` file to
version control. It should only include environment-specific values
such as database passwords or API keys. Your production database
should have a different password than your development database.

#### Should I commit my `.env.vault` file?

Yes. It is safe and recommended to do so. It contains your encrypted
envs, and your vault identifier.

#### Can I share the `DOTENV_KEY`?

No. It is the key that unlocks your encrypted environment variables.
Be very careful who you share this key with. Do not let it leak.

## Contributing

1. Fork it
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Added some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create new Pull Request

## Changelog

See [CHANGELOG.md](CHANGELOG.md)

## License

MIT
