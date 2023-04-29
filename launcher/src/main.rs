#![feature(exit_status_error)]

use clap::{Parser, ValueEnum};
use dll_syringe::{error::InjectError, process::OwnedProcess, Syringe};
use std::io;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, ExitStatusError, Stdio};
use thiserror::Error;
const BACKEND_ERRORS: [&str; 5] = [
    "port 3551 failed: Connection refused",
    "Unable to login to Fortnite servers",
    "HTTP 400 response from ",
    "Network failure when attempting to check platform restrictions",
    "UOnlineAccountCommon::ForceLogout",
];

const SERVER_ORIGINAL: [u8; 112] = [
    45, 0, 105, 0, 110, 0, 118, 0, 105, 0, 116, 0, 101, 0, 115, 0, 101, 0, 115, 0, 115, 0, 105, 0,
    111, 0, 110, 0, 32, 0, 45, 0, 105, 0, 110, 0, 118, 0, 105, 0, 116, 0, 101, 0, 102, 0, 114, 0,
    111, 0, 109, 0, 32, 0, 45, 0, 112, 0, 97, 0, 114, 0, 116, 0, 121, 0, 95, 0, 106, 0, 111, 0,
    105, 0, 110, 0, 105, 0, 110, 0, 102, 0, 111, 0, 95, 0, 116, 0, 111, 0, 107, 0, 101, 0, 110, 0,
    32, 0, 45, 0, 114, 0, 101, 0, 112, 0, 108, 0, 97, 0, 121, 0,
];

const SERVER_PATCHED: [u8; 112] = [
    45, 0, 108, 0, 111, 0, 103, 0, 32, 0, 45, 0, 110, 0, 111, 0, 115, 0, 112, 0, 108, 0, 97, 0,
    115, 0, 104, 0, 32, 0, 45, 0, 110, 0, 111, 0, 115, 0, 111, 0, 117, 0, 110, 0, 100, 0, 32, 0,
    45, 0, 110, 0, 117, 0, 108, 0, 108, 0, 114, 0, 104, 0, 105, 0, 32, 0, 45, 0, 117, 0, 115, 0,
    101, 0, 111, 0, 108, 0, 100, 0, 105, 0, 116, 0, 101, 0, 109, 0, 99, 0, 97, 0, 114, 0, 100, 0,
    115, 0, 32, 0, 32, 0, 32, 0, 32, 0, 32, 0, 32, 0, 32, 0,
];

const MATCHMAKING_ORIGINAL: [u8; 33] = [
    63, 0, 69, 0, 110, 0, 99, 0, 114, 0, 121, 0, 112, 0, 116, 0, 105, 0, 111, 0, 110, 0, 84, 0,
    111, 0, 107, 0, 101, 0, 110, 0, 61,
];

const MATCHMAKING_PATCHED: [u8; 33] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];

/// Fortnite launcher
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to fortnite build.
    #[arg(short, long)]
    build_path: String,

    #[clap(value_enum, default_value_t=LaunchType::Client)]
    launch_type: LaunchType,

    /// Path to dlls directory.
    #[arg(short, long)]
    dlls_path: String,
}

#[derive(Error, Debug)]
enum LauncherError {
    #[error("Failed to spawn fortnite executable: `{0}`")]
    SpawnFailed(io::Error),

    #[error("Failed to patch fortnite executable: `{0}`")]
    PatchFailed(#[from] PatchError),

    #[error("Error while altering fortnite process: `{0}`")]
    HandleChildError(#[from] HandleChildError),

    #[error("`{0}`")]
    ExitStatusError(#[from] ExitStatusError),
}

#[derive(Error, Debug)]
enum PatchError {
    #[error("Couldn't read original fortnite executable: `{0}`")]
    ReadError(io::Error),

    #[error("Couldn't write patched fortnite executable: `{0}`")]
    WriteError(io::Error),
}

#[derive(Error, Debug)]
enum HandleChildError {
    #[error("Failed create syringe for fortnite process: `{0}`")]
    SyringeFailed(io::Error),

    #[error("Unable to get stdout of fortnite process")]
    StdoutClosed,

    #[error("Could not connect to the backend: `{0}`")]
    BackendConnectionError(String),

    #[error("Could not inject to fortnite process: `{0}`")]
    InjectError(#[from] InjectError),
}

#[derive(ValueEnum, Clone, Debug, Eq, PartialEq)]
enum LaunchType {
    Client,
    Server,
}

fn main() -> Result<(), LauncherError> {
    let args = Args::parse();

    launch_fortnite(
        Path::new(&args.build_path),
        Path::new(&args.dlls_path),
        &args.launch_type,
    )?
    .exit_ok()?;

    Ok(())
}

fn launch_fortnite(
    build_path: &Path,
    dlls_path: &Path,
    launch_type: &LaunchType,
) -> Result<ExitStatus, LauncherError> {
    let mut command = Command::new(patch_fortnite(build_path, launch_type)?);
    command.stdout(Stdio::piped())
        .args([
            "-epicapp=Fortnite",
            "-epicenv=Prod",
            "-epiclocale=en-us",
            "-epicportal",
            "-skippatchcheck",
            "-nobe",
            "-fromfl=eac",
            "-fltoken=3db3ba5dcbd2e16703f3978d",
            "-caldera=eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2NvdW50X2lkIjoiYmU5ZGE1YzJmYmVhNDQwN2IyZjQwZWJhYWQ4NTlhZDQiLCJnZW5lcmF0ZWQiOjE2Mzg3MTcyNzgsImNhbGRlcmFHdWlkIjoiMzgxMGI4NjMtMmE2NS00NDU3LTliNTgtNGRhYjNiNDgyYTg2IiwiYWNQcm92aWRlciI6IkVhc3lBbnRpQ2hlYXQiLCJub3RlcyI6IiIsImZhbGxiYWNrIjpmYWxzZX0.VAWQB67RTxhiWOxx7DBjnzDnXyyEnX7OljJm-j2d88G_WgwQ9wrE6lwMEHZHjBd1ISJdUO1UVUqkfLdU5nofBQ",
            "-AUTH_LOGIN=reboot@projectreboot.dev",
            "-AUTH_PASSWORD=Rebooted",
            "-AUTH_TYPE=epic"
        ]);

    if launch_type.eq(&LaunchType::Server) {
        command.args(["-nullrhi", "-nosplash", "-nosound"]);
    }

    let mut bin_child = command.spawn().map_err(LauncherError::SpawnFailed)?;

    if let Err(e) = handle_child(&mut bin_child, dlls_path) {
        let _ = bin_child.kill();
        Err(e)?
    }
    bin_child.wait().map_err(LauncherError::SpawnFailed)
}

fn patch_fortnite(build_path: &Path, launch_type: &LaunchType) -> Result<PathBuf, PatchError> {
    let binaries_path = build_path
        .join("FortniteGame")
        .join("Binaries")
        .join("Win64/");
    let binary = binaries_path.join("FortniteClient-Win64-Shipping.exe");

    let mut new_exec_data = std::fs::read(&binary).map_err(PatchError::ReadError)?;

    if launch_type.eq(&LaunchType::Server) {
        replace_slice(&mut new_exec_data[..], &SERVER_ORIGINAL, &SERVER_PATCHED)
    }
    replace_slice(
        &mut new_exec_data[..],
        &MATCHMAKING_ORIGINAL,
        &MATCHMAKING_PATCHED,
    );

    std::fs::write(&binary, new_exec_data).map_err(PatchError::WriteError)?;

    Ok(binary)
}

fn handle_child(bin_child: &mut Child, dlls_path: &Path) -> Result<(), HandleChildError> {
    let syringe = Syringe::for_process(
        OwnedProcess::from_pid(bin_child.id()).map_err(HandleChildError::SyringeFailed)?,
    );

    let bin_out = bin_child
        .stdout
        .take()
        .ok_or(HandleChildError::StdoutClosed)?;
    for line in BufReader::new(bin_out).lines() {
        let Ok(line) = line else { continue; };
        println!("{line}");
        if line.contains("Platform has ") {
            syringe.inject(dlls_path.join("craniumv2.dll"))?;
        } else if line.contains("Region ") {
            syringe.inject(dlls_path.join("server.dll"))?;
            syringe.inject(dlls_path.join("leakv2.dll"))?;
        } else if BACKEND_ERRORS.iter().any(|e| line.contains(e))
            || line.contains("FOnlineSubsystemGoogleCommon::Shutdown()")
        {
            return Err(HandleChildError::BackendConnectionError(line));
        }
    }
    Ok(())
}

fn replace_slice<T, const N: usize>(source: &mut [T], from: &[T; N], to: &[T; N])
where
    T: Clone + PartialEq,
{
    for i in 0..=source.len() - from.len() {
        if source[i..].starts_with(from) {
            source[i..(i + from.len())].clone_from_slice(to);
        }
    }
}
