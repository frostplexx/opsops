use std::{ffi::OsString, path::Path};

use colored::Colorize;

use crate::util::{
    print_status::{print_error, print_info, print_success},
    sops_command::SopsCommandBuilder,
    sops_status::is_file_unchanged_status,
};

pub fn read(path: OsString) {
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

    let sops_command = match SopsCommandBuilder::new()
        .arg("-d")
        .arg(&path_str)
        .with_age_key()
    {
        Ok(cmd) => cmd,
        Err(e) => {
            print_error(format!("{} {}", "Failed to get Age key:".red(), e));
            std::process::exit(1);
        }
    };

    match sops_command.status() {
        Ok(status) => {
            std::process::exit(status.code().unwrap_or(1));
        }
        Err(e) => {
            print_error(format!("{} {:?}", "Failed to launch sops:".red(), e));
            std::process::exit(1);
        }
    }
}
