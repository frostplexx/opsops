mod commands;
mod util;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate_to, shells::Fish};
use clap_mangen::Man;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path;
use std::path::Path;
use util::print_status::print_info;

#[derive(Debug, Parser)]
#[command(name = "opsops")]
#[command(version, about = "A wrapper that integrates sops with 1Password", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Parse and display the .sops.yaml for this project
    #[command(arg_required_else_help = false)]
    ListConfig {},

    /// Generate an age key pair
    #[command(arg_required_else_help = false)]
    GenerateAgeKey {},

    /// Edit a file using sops with a key from 1password
    #[command(arg_required_else_help = true)]
    Edit {
        #[arg(value_name = "PATH", help = "Path to the file to edit")]
        path: OsString,
    },

    /// Encrypt a file using sops
    #[command(arg_required_else_help = true)]
    Encrypt {
        #[arg(value_name = "PATH", help = "Path to the file to encrypt")]
        path: OsString,
    },

    /// Decrypt a file using sops
    #[command(arg_required_else_help = true)]
    Decrypt {
        #[arg(value_name = "PATH", help = "Path to the encrypted file to decrypt")]
        path: OsString,
    },

    /// Troubleshoot your current config
    #[command(arg_required_else_help = false)]
    Doctor {},

    /// Initialize opsops
    Init {},

    /// Read an encrypted file and print its decrypted content to stdout
    Read {
        #[arg(value_name = "PATH", help = "Path to the file to read")]
        path: OsString,
    },

    /// Set up encryption patterns for a file
    #[command(arg_required_else_help = true)]
    TargetKeys {
        #[arg(
            value_name = "PATH",
            help = "Path to the file to configure encryption for"
        )]
        path: OsString,
    },

    /// Generate shell completions and man pages
    #[command(arg_required_else_help = false, hide = true)]
    GenerateDocs {
        /// Output directory for generated documentation
        #[arg(short, long, default_value = "target/doc")]
        dir: String,
    },
}

impl Cli {
    /// Generate man pages and shell completions
    fn generate_docs(output_dir: &str) -> io::Result<()> {
        let out_dir = Path::new(output_dir);
        let man_dir = out_dir.join("man");
        let completion_dir = out_dir.join("completions");

        // Create directories if they don't exist
        fs::create_dir_all(&man_dir)?;
        fs::create_dir_all(&completion_dir)?;

        // Generate man page
        let cmd = Cli::command();
        let man = Man::new(cmd.clone());
        let mut buffer = Vec::<u8>::new();
        man.render(&mut buffer)?;

        let man_path = man_dir.join("opsops.1");
        fs::write(man_path, buffer)?;
        println!(
            "Generated man page at: {}",
            man_dir.join("opsops.1").display()
        );

        // Generate Fish completions
        let mut cmd = Cli::command();
        let path = generate_to(Fish, &mut cmd, "opsops", &completion_dir)?;
        print_info(format!("Generated Fish completions at: {}", path.display()));

        println!("\nTo install:");
        println!("  Man pages:          mkdir -p ~/.local/share/man/man1");
        println!(
            "                      cp {}/opsops.1 ~/.local/share/man/man1/",
            man_dir.display()
        );
        println!("                      mandb  # Update man database");
        println!("  Fish completions:   mkdir -p ~/.config/fish/completions");
        println!(
            "                      cp {}/opsops.fish ~/.config/fish/completions/",
            completion_dir.display()
        );

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::ListConfig {} => commands::list_config::list_config(),
        Commands::GenerateAgeKey {} => commands::generate_age_key::generate_age_key(),
        Commands::Edit { path } => commands::edit::edit(path),
        Commands::Encrypt { path } => commands::encrypt::encrypt(path),
        Commands::Decrypt { path } => commands::decrypt::decrypt(path),
        Commands::Init {} => commands::init::init(),
        Commands::Doctor {} => commands::doctor::doctor(),
        Commands::TargetKeys { path } => commands::set_key::set_keys(path),
        Commands::GenerateDocs { dir } => Cli::generate_docs(&dir)?,
        Commands::Read { path } => commands::read::read(path),
    }

    Ok(())
}
