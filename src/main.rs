mod builder;
mod config;
mod grub;

use std::{os::linux::fs::MetadataExt, path::Path};

use eyre::{bail, Result};
use roxmltree::Document;

use crate::{builder::Builder, config::Config};

fn main() -> Result<()> {
	color_eyre::install()?;

	let mut args = std::env::args();
	let Some(config_file) = args.nth(1) else {
		bail!("Config file not given: expected it to be the first argument")
	};
	let Some(default_config) = args.next() else {
		bail!("Default config not given: expected it to be the second argument")
	};

	// For debugging purposes

	let document_file = std::fs::read_to_string(config_file)?;
	let document = Document::parse(&document_file)?;

	let mut config = Config::new(&document)?;

	// Discover whether the bootPath is on the same filesystem as / and
	// /nix/store.  If not, then all kernels and initrds must be copied to
	// the bootPath.
	if config.boot_path.metadata()?.st_dev() != Path::new("/nix/store").metadata()?.st_dev() {
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
