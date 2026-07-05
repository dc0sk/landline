//! Operations helper: generate an argon2 password hash for a `config.toml` user.
//!
//! Usage:
//!   cargo run -p landline-backend --bin landline-hash -- <password>
//!
//! Copy the printed PHC string into a `[[auth.users]]` `password_hash` field
//! (note the table path — users live under `[auth]`). The plaintext password is
//! never stored (NFR-SEC-12).

use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(password) = std::env::args().nth(1) else {
        eprintln!("usage: landline-hash <password>");
        return ExitCode::FAILURE;
    };
    let Ok(hash) = landline_backend::auth::hash_password(&password) else {
        eprintln!("failed to hash password");
        return ExitCode::FAILURE;
    };
    println!("{hash}");
    ExitCode::SUCCESS
}
