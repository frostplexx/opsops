use crate::{GlobalContext, util::op_key::get_age_key_from_1password};
use std::process::{Child, Command, Stdio};

/// A helper type for executing SOPS commands with the Age key from 1Password
pub struct SopsCommandBuilder<'a> {
    command: Command,
    has_age_key: bool,
    context: &'a GlobalContext,
}

impl<'a> SopsCommandBuilder<'a> {
    /// Create a new SopsCommandBuilder initialized with the sops binary
    pub fn new(context: &'a GlobalContext) -> Self {
        let mut command = Command::new("sops");

        // If a custom sops file is specified, add the --config flag
        if let Some(sops_file) = &context.sops_file {
            command.arg("--config").arg(sops_file);
        }

        SopsCommandBuilder {
            command,
            has_age_key: false,
            context,
        }
    }

    /// Add an argument to the SOPS command
    pub fn arg<S: AsRef<std::ffi::OsStr>>(mut self, arg: S) -> Self {
        self.command.arg(arg);
        self
    }

    /// Add multiple arguments to the SOPS command
    pub fn _args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.command.args(args);
        self
    }

    /// Set the working directory for the command
    pub fn _current_dir<P: AsRef<std::path::Path>>(mut self, dir: P) -> Self {
        self.command.current_dir(dir);
        self
    }

    /// Configure with Age key from 1Password (if it exists)
    pub fn with_age_key(mut self) -> Result<Self, String> {
        // Retrieve the Age key from 1Password
        let age_key = get_age_key_from_1password(self.context)?;
        self.command.env("SOPS_AGE_KEY", age_key);
        self.has_age_key = true;
        Ok(self)
    }

    /// Try to set the Age key, but don't fail if it's not available
    pub fn _with_optional_age_key(mut self) -> Self {
        if let Ok(age_key) = get_age_key_from_1password(self.context) {
            self.command.env("SOPS_AGE_KEY", age_key);
            self.has_age_key = true;
        }
        self
    }

    /// Run the command and wait for it to finish
    pub fn status(mut self) -> std::io::Result<std::process::ExitStatus> {
        self.command.status()
    }

    /// Spawn the command and return the Child process handle
    pub fn _spawn(mut self) -> std::io::Result<Child> {
        self.command.spawn()
    }

    /// Run the command and capture its output
    pub fn _output(mut self) -> std::io::Result<std::process::Output> {
        self.command.output()
    }

    /// Check if the Age key was successfully set
    pub fn _has_age_key(&self) -> bool {
        self.has_age_key
    }

    /// Set stdin for the command
    pub fn _stdin(mut self, cfg: Stdio) -> Self {
        self.command.stdin(cfg);
        self
    }

    /// Set stdout for the command
    pub fn _stdout(mut self, cfg: Stdio) -> Self {
        self.command.stdout(cfg);
        self
    }

    /// Set stderr for the command
    pub fn _stderr(mut self, cfg: Stdio) -> Self {
        self.command.stderr(cfg);
        self
    }
}

#[cfg(test)]
mod tests {

    use std::process::Stdio;

    use crate::GlobalContext;
    use crate::util::sops_command::SopsCommandBuilder;

    fn mock_context(opitem: Option<String>) -> GlobalContext {
        GlobalContext {
            opitem,
            sops_file: None,
        }
    }

    #[test]
    fn test_builder_runs_valid_command() {
        if which::which("sops").is_err() {
            eprintln!("Skipping test_builder_runs_valid_command: 'sops' binary not found in PATH.");
            return;
        }
        let context = mock_context(None);

        let output = SopsCommandBuilder::new(&context)
            .arg("--version")
            ._output()
            .expect("Failed to run sops");

        assert!(output.status.success());
        let out_str = String::from_utf8_lossy(&output.stdout);
        assert!(
            out_str.contains("sops") || out_str.contains("version"),
            "unexpected output: {}",
            out_str
        );
    }

    #[test]
    fn test_env_injection() {
        if which::which("sops").is_err() {
            eprintln!("Skipping test_env_injection: 'sops' binary not found in PATH.");
            return;
        }
        let context = mock_context(Some(
            "AGE-SECRET-KEY-1AM036DUJQ8RTJ84N7JTJECSV6FXFM3DCM9F4VEX4ZPL4M3VDA6FQLVJSUR"
                .to_string(),
        ));

        let output = SopsCommandBuilder::new(&context)
            ._with_optional_age_key()
            .arg("-e")
            .arg("/dev/null")
            ._stderr(Stdio::piped())
            ._output();

        match output {
            Ok(output) => {
                // Not checking for success, just that command ran and Age key was accepted
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!stderr.contains("missing AGE key"));
            }
            Err(e) => panic!("Command execution failed: {}", e),
        }
    }
}
