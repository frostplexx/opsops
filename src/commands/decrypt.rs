use crate::util::print_status::{print_error, print_info, print_success};
use crate::util::sops_command::SopsCommandBuilder;
use crate::util::sops_status::is_file_unchanged_status;
use colored::Colorize;
use std::ffi::OsString;
use std::path::Path;

/// Decrypts a file using SOPS with the Age key from 1Password
pub fn decrypt(path: OsString) {
    // Convert the path from OsString to String
    let path_str = match path.into_string() {
        Ok(p) => p,
        Err(os) => {
            print_error(format!("{} {:?}", "Invalid UTF-8 in path:".red(), os));
            std::process::exit(1);
        }
    };

    // Check if the file exists
    if !Path::new(&path_str).is_file() {
        print_error(format!("{} {}", "File not found:".red(), path_str));
        std::process::exit(1);
    }

    // Ensure sops is installed
    if which::which("sops").is_err() {
        print_error(format!(
            "{} {}",
            "'sops' is not installed or not in PATH.".red(),
            "Please install it first.".dimmed()
        ));
        std::process::exit(1);
    }

    // Create the decrypted output path - remove .enc extension if it exists, otherwise add .dec
    let output_path = if path_str.ends_with(".enc") {
        path_str[..path_str.len() - 4].to_string()
    } else {
        format!("{}", path_str)
    };

    println!(
        "{} {} {} {}",
        "🔓 Decrypting".green(),
        path_str,
        "to".green(),
        output_path
    );

    // Create a SOPS command with the Age key from 1Password
    let sops_command = match SopsCommandBuilder::new()
        .arg("--decrypt")
        .arg("--output")
        .arg(&output_path)
        .arg(&path_str)
        .with_age_key()
    {
        Ok(cmd) => cmd,
        Err(e) => {
            print_error(format!("{} {}", "Failed to get Age key:".red(), e));
            std::process::exit(1);
        }
    };

    // Run the command
    match sops_command.status() {
        Ok(status) if status.success() => {
            print_success(format!(
                "{} {} {}",
                "Successfully decrypted file to".green(),
                output_path,
                "with SOPS".green()
            ));
        }
        Ok(status) if is_file_unchanged_status(&status) => {
            print_info(format!(
                "{} {}",
                "File has not changed.".blue(),
                output_path
            ));
        }
        Ok(status) => {
            print_error(format!(
                "{} Exit code: {}",
                "Error while decrypting the file.".red(),
                status
            ));
            std::process::exit(status.code().unwrap_or(1));
        }
        Err(e) => {
            print_error(format!("{} {:?}", "Failed to launch sops:".red(), e));
            std::process::exit(1);
        }
    }
}
