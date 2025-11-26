use safe_cargo::prepare_profile;
use std::{
    collections::VecDeque,
    env, fs, io,
    process::{Command, ExitCode},
};

/// This crate is available only on macOS becuase it relies on `sandbox-exec` cli tool
#[cfg(target_os = "macos")]
fn main() -> Result<ExitCode, io::Error> {
    let mut args = env::args().collect::<VecDeque<_>>();

    let _program_name = args.pop_front();

    let Ok(workspace_path) = env::current_dir() else {
        panic!("Error reading current directory");
    };

    let sandbox_path = workspace_path.join(".sandbox");
    if !fs::exists(&sandbox_path)? {
        fs::create_dir_all(&sandbox_path)?;
    }

    let sandbox_profile = prepare_profile(&workspace_path, &sandbox_path)?;
    if args.iter().any(|o| *o == "--dump-profile") {
        println!("{}", sandbox_profile);
        return Ok(ExitCode::SUCCESS);
    }
    let profile_path = sandbox_path.join("profile.sb");
    fs::write(&profile_path, sandbox_profile.to_string())?;

    let result = Command::new("sandbox-exec")
        .arg("-f")
        .arg(profile_path)
        .arg("cargo")
        .args(args)
        .env("CARGO_TARGET_DIR", sandbox_path.join("target"))
        .env("CARGO_HOME", sandbox_path.join("cargo"))
        .spawn()?
        .wait()?;

    let code = match result.code() {
        Some(0) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    };
    Ok(code)
}
