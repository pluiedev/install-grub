mod builder;
mod config;
mod grub;

use std::{os::linux::fs::MetadataExt, path::Path};

use anyhow::{anyhow, bail, Result};
use roxmltree::Document;

use crate::{builder::Builder, config::Config};

fn main() -> Result<()> {
	let mut args = std::env::args();
	let Some(config_file) = args.next() else {
		bail!("Config file not given: expected it to be the first argument")
	};
	let Some(default_config) = args.next() else {
		bail!("Default config not given: expected it to be the second argument")
	};

	let document_file = std::fs::read_to_string(config_file)?;
	let document = Document::parse(&document_file)?;

	// The manual anyhow wrap was because the error's lifetime is pinned to the
	// document, so the error could not be thrown outside of the function without
	// converting to a plain string first
	let mut config = Config::new(&document).map_err(|e| anyhow!("{e}"))?;

	// Discover whether the bootPath is on the same filesystem as / and
	// /nix/store.  If not, then all kernels and initrds must be copied to
	// the bootPath.
	if std::fs::metadata(config.boot_path)?.st_dev() != std::fs::metadata("/nix/store")?.st_dev() {
		config.copy_kernels = true;
	}

	eprintln!("updating GRUB 2 menu...");

	std::env::set_var("PATH", config.path);

	Builder::new(config, Path::new(&default_config))?
		.users()?
		.default_entry()?
		.appearance()?
		.entries()?
		.install()?;

	Ok(())
}
