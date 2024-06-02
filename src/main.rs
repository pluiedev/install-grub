mod builder;
mod config;
mod grub;
mod install;

use std::{os::linux::fs::MetadataExt, path::PathBuf};

use anyhow::{bail, Result};

use crate::{builder::Builder, config::Config};

fn main() -> Result<()> {
	let mut args = std::env::args();
	let Some(config_file) = args.next() else {
		bail!("Config file not given: expected it to be the first argument")
	};
	let Some(default_config) = args.next() else {
		bail!("Default config not given: expected it to be the second argument")
	};
	let mut config = Config::new(&config_file)?;

	// Discover whether the bootPath is on the same filesystem as / and
	// /nix/store.  If not, then all kernels and initrds must be copied to
	// the bootPath.
	if std::fs::metadata(&config.boot_path)?.st_dev() != std::fs::metadata("/nix/store")?.st_dev() {
		config.copy_kernels = true;
	}

	eprintln!("updating GRUB 2 menu...");

	std::env::set_var("PATH", &config.path);

	let mut builder = Builder::new(&config, PathBuf::from(default_config))?;
	let (conf, temp) = builder
		.append_users()?
		.append_default_entry()?
		.append_font()?
		.append_splash()?
		.append_theme()?
		.append_extra_config()?
		.append_default_entries()?
		.append_profiles()?
		.append_prepare_config()?
		.write()?;

	install::install(&conf, &temp, &builder.copied, &config)?;

	Ok(())
}
