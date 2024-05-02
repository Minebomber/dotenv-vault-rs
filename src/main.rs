use argh::FromArgs;
use std::env;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

#[derive(FromArgs, PartialEq, Debug)]
/// The CLI program to load the .env.vault file and run the specified program with the specified arguments.
///
/// You have to set the DOTENV_KEY environment variable before calling dotenv-vault.
///
/// Example:
/// dotenv-vault run -- my_program arg1 arg2
struct Opts {
    #[argh(subcommand)]
    commands: Commands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Commands {
    Run(Run),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Load the .env.vault file and run the specified program with the specified arguments.
#[argh(subcommand, name = "run")]
struct Run {
    #[argh(switch, long = "override")]
    /// whether to override the existing environment variables
    override_: bool,

    #[argh(option)]
    /// current working directory to run the program in
    cwd: Option<PathBuf>,

    #[argh(positional)]
    /// the program to run
    program: String,

    #[argh(positional)]
    /// the arguments to pass to the program
    program_args: Vec<String>,
}

#[derive(Debug)]
#[repr(i32)]
enum CLIError {
    EnvLoad = 1,
    EnvOverrideLoad = 2,
    ProgramExecution = 3,
    CwdChange = 4,
}

fn main() {
    let opts = argh::from_env::<Opts>();

    match opts.commands {
        Commands::Run(run_opts) => {
            let current_cwd = env::current_dir().unwrap();

            if let Some(given_cwd) = run_opts.cwd {
                env::set_current_dir(&given_cwd).unwrap_or_else(|err| {
                    eprintln!("Failed to change the current working directory: {}", err);
                    exit(CLIError::CwdChange as i32);
                });
            }

            // Load the .env.vault file
            if run_opts.override_ {
                dotenv_vault::dotenv_override().unwrap_or_else(|err| {
                    eprintln!("Failed to load env: {}", err);
                    exit(CLIError::EnvOverrideLoad as i32);
                });
            } else {
                dotenv_vault::dotenv().unwrap_or_else(|err| {
                    eprintln!("Failed to load env: {}", err);
                    exit(CLIError::EnvLoad as i32);
                });
            };

            // Run the specified program with the specified arguments
            let output = Command::new(&run_opts.program)
                .args(run_opts.program_args)
                .envs(env::vars())
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .unwrap_or_else(|err| {
                    eprintln!("Failed to execute program {}: {}", run_opts.program, err);
                    exit(CLIError::ProgramExecution as i32);
                });

            // Restore the current working directory
            env::set_current_dir(current_cwd).unwrap_or_else(|err| {
                eprintln!("Failed to change the current working directory: {}", err);
                exit(CLIError::CwdChange as i32);
            });

            if !output.status.success() {
                exit(
                    output
                        .status
                        .code()
                        .unwrap_or(CLIError::ProgramExecution as i32),
                );
            }
        }
    }
}
