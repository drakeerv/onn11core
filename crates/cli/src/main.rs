use clap::{Parser, Subcommand};
use onn11core_core::{self as core, Settings, StdoutReporter, Variant};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "onn11core",
    version,
    about = "Unlock, back up and install LineageOS on the onn 11\" Core (T405A)"
)]
struct Cli {
    /// Data directory for images, logs and backups
    #[arg(long, global = true)]
    workdir: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show device and tool state
    Check,
    /// Pre-fetch images
    Download {
        #[arg(value_parser = ["trebledroid", "vanilla", "microg", "gapps", "all"])]
        what: String,
    },
    /// Unlock the bootloader (wipes userdata)
    Unlock {
        #[arg(long)]
        yes: bool,
    },
    /// Back up the stock partitions through a DSU userdebug GSI
    Backup {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Flash a GSI
    Flash {
        #[arg(long)]
        backup: PathBuf,
        /// Custom raw ext4 arm64 GSI
        #[arg(long)]
        image: Option<PathBuf>,
        #[arg(long, value_parser = ["vanilla", "microg", "gapps"], default_value = "vanilla")]
        variant: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Flash the recommended Google GKI boot image
    Gki {
        #[arg(long)]
        image: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Patch init_boot with Magisk and flash it
    Root {
        #[arg(long)]
        backup: PathBuf,
        #[arg(long)]
        patched: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Restore stock boot/vbmeta/super from a backup
    Restore {
        #[arg(long)]
        backup: PathBuf,
        #[arg(long)]
        no_super: bool,
        /// Restore logical partitions in fastbootd with lpunpack instead of super.img
        #[arg(long)]
        logical: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// check -> unlock -> backup -> flash -> GKI
    All {
        #[arg(long, value_parser = ["vanilla", "microg", "gapps"], default_value = "vanilla")]
        variant: String,
        /// Keep the stock kernel instead of the recommended GKI
        #[arg(long)]
        stock_kernel: bool,
        #[arg(long)]
        yes: bool,
    },
}

fn default_workdir() -> PathBuf {
    dirs::data_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("onn11core")
}

fn variant(value: &str) -> anyhow::Result<Variant> {
    Variant::parse(value).ok_or_else(|| anyhow::anyhow!("unknown variant: {value}"))
}

fn confirm_unlock() -> bool {
    print!("Type exactly: UNLOCK ONN11CORE\n> ");
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return false;
    }
    line.trim() == "UNLOCK ONN11CORE"
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let settings = Settings::new(cli.workdir.unwrap_or_else(default_workdir));
    let rep = StdoutReporter;

    match cli.cmd {
        Cmd::Check => {
            core::check(&rep)?;
        }
        Cmd::Download { what } => match what.as_str() {
            "trebledroid" => {
                core::download::ensure_trebledroid(&rep, &settings)?;
            }
            "vanilla" => {
                core::download::ensure_lineage(&rep, &settings, Variant::Vanilla)?;
            }
            "microg" => {
                core::download::ensure_lineage(&rep, &settings, Variant::MicroG)?;
            }
            "gapps" => {
                core::download::ensure_lineage(&rep, &settings, Variant::Gapps)?;
            }
            _ => core::download_all(&rep, &settings)?,
        },
        Cmd::Unlock { yes } => {
            if !yes && !confirm_unlock() {
                anyhow::bail!("confirmation did not match; nothing done");
            }
            core::unlock::unlock(&rep, &settings)?;
        }
        Cmd::Backup { out } => {
            core::backup::backup(&rep, &settings, out.as_deref())?;
        }
        Cmd::Flash {
            backup,
            image,
            variant: v,
            force,
            dry_run,
        } => {
            core::flash::flash(
                &rep,
                &settings,
                &backup,
                image.as_deref(),
                variant(&v)?,
                force,
                dry_run,
            )?;
        }
        Cmd::Gki { image, dry_run } => {
            core::flash::flash_gki(&rep, &settings, image.as_deref(), dry_run)?;
        }
        Cmd::Root {
            backup,
            patched,
            dry_run,
        } => {
            core::magisk::install(&rep, &settings, &backup, patched.as_deref(), dry_run)?;
        }
        Cmd::Restore {
            backup,
            no_super,
            logical,
            dry_run,
        } => {
            core::flash::restore(&rep, &settings, &backup, no_super, logical, dry_run)?;
        }
        Cmd::All {
            variant: v,
            stock_kernel,
            yes,
        } => {
            let variant = variant(&v)?;
            core::check(&rep)?;
            let summary = core::device::require_model(&rep)?;
            if summary.locked != "0" {
                if !yes && !confirm_unlock() {
                    anyhow::bail!("confirmation did not match; nothing done");
                }
                core::unlock::unlock(&rep, &settings)?;
            }
            let backup = core::backup::backup(&rep, &settings, None)?;
            core::flash::flash(&rep, &settings, &backup, None, variant, false, false)?;
            if !stock_kernel {
                core::flash::flash_gki(&rep, &settings, None, false)?;
            }
        }
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("ERROR: {err:#}");
        std::process::exit(1);
    }
}
