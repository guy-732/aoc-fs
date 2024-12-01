use std::{path::PathBuf, process::ExitCode};

use clap::{
    builder::styling::{AnsiColor, Color, Style},
    Parser,
};
use fuser::{mount2, MountOption};

mod config;
mod filesystem;

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
}

#[derive(Debug, clap::Parser)]
#[command(styles = get_styles())]
struct CmdArgs {
    #[arg(help = "Act as a client, and mount FUSE at given path")]
    mount_point: PathBuf,

    #[arg(
        short,
        long,
        help = "TOML file containing configuration",
        default_value = "aoc-fs-config.toml"
    )]
    config_file: PathBuf,

    #[arg(long, help = "Automatically unmount on process exit")]
    auto_unmount: bool,

    #[arg(long, help = "Allow root user to access the filesystem")]
    allow_root: bool,

    #[arg(long, help = "Allow all users to access the filesystem")]
    allow_other: bool,
}

fn main() -> ExitCode {
    let args = CmdArgs::parse();
    pretty_env_logger::init();

    let mut mount_options = Vec::from_iter([
        MountOption::NoExec,
        MountOption::NoSuid,
        MountOption::NoDev,
        MountOption::NoAtime,
        MountOption::RO,
        MountOption::DefaultPermissions,
        MountOption::FSName("aoc-fs".into()),
    ]);

    if args.allow_other {
        mount_options.push(MountOption::AllowOther);
    }

    if args.allow_root {
        mount_options.push(MountOption::AllowRoot);
    }

    if args.auto_unmount {
        mount_options.push(MountOption::AutoUnmount);
    }

    let config = match config::Config::load_config(&args.config_file) {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!(
                "Failed to load configuration file {:?}: {}",
                args.config_file, e
            );
            return ExitCode::FAILURE;
        }
    };

    let fs = filesystem::AoCFilesystem::new(config);
    log::trace!("Mounting fs on {:?}", args.mount_point);
    match mount2(fs, args.mount_point, &mount_options) {
        Ok(()) => {
            log::info!("Filesystem unmounted");
            ExitCode::SUCCESS
        }
        Err(e) => {
            log::error!("mount error: {}", e);
            ExitCode::FAILURE
        }
    }
}
