//! Explicit, read-only Cargo release-candidate rendering.

#[cfg(unix)]
mod release_metadata_cli;

fn main() -> std::process::ExitCode {
    #[cfg(unix)]
    match release_metadata_cli::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Cargo release candidate refused: {error}");
            std::process::ExitCode::from(2)
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!("Cargo release candidate reads require Unix no-follow admission");
        std::process::ExitCode::from(2)
    }
}
