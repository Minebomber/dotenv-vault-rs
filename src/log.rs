use std::fmt::Display;

/// Macro for generating logging functions
macro_rules! log_fn {
    ($name:tt, $level:tt) => {
        pub fn $name<T>(message: T)
        where
            T: Display,
        {
            eprintln!(
                "[dotenv-vault@{}][{}] {}",
                env!("CARGO_PKG_VERSION"),
                $level,
                message
            );
        }
    };
}

log_fn!(info, "INFO");
log_fn!(warn, "WARN");
