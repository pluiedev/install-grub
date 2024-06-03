use std::{collections::HashSet, fmt::Write as _, fs, path::PathBuf};

use eyre::{bail, Result, WrapErr};
use walkdir::WalkDir;

use super::Builder;
use crate::config::Config;

impl Builder<'_> {
	pub fn appearance(&mut self) -> Result<&mut Self> {
		self.append_font()?;
		self.append_splash()?;
		self.append_theme()?;
		self.append_extra_config()?;

		Ok(self)
	}

	pub fn append_font(&mut self) -> Result<()> {
		let Config {
			font,
			boot_path,
			gfx_mode_efi,
			gfx_payload_efi,
			gfx_mode_bios,
			gfx_payload_bios,
			..
		} = &self.config;

		fs::copy(font, boot_path.join("converted-font.pf2")).with_context(|| {
			format!("Cannot copy {} to {}", font.display(), boot_path.display())
		})?;

		write!(
			&mut self.inner,
			r#"insmod font
if loadfont "{}"/converted-font.pf2; then
  insmod gfxterm
  if [ "${{grub_platform}}" = "efi" ]; then
    set gfxmode={gfx_mode_efi}
    set gfxpayload={gfx_payload_efi}
  else
    set gfxmode={gfx_mode_bios}
    set gfxpayload={gfx_payload_bios}
  fi
  terminal_output gfxterm
fi
"#,
			self.grub_boot_path_normalized.display(),
		)?;

		Ok(())
	}

	pub fn append_splash(&mut self) -> Result<()> {
		let Config {
			splash_image,
			background_color,
			boot_path,
			splash_mode,
			..
		} = &self.config;

		let Some(splash_image) = splash_image else {
			return Ok(());
		};

		let mut target = PathBuf::from("background");

		let ext = if let Some(ext) = splash_image.extension() {
			if ext == "jpg" {
				"jpeg".into()
			} else {
				ext.to_string_lossy()
			}
		} else {
			bail!("Splash image has no extension - could not decide which module to load!");
		};

		target.set_extension(ext.as_ref());

		if let Some(background_color) = background_color {
			write!(&mut self.inner, "background_color '{background_color}'")?;
		}

		fs::copy(splash_image, boot_path.join(&target)).with_context(|| {
			format!(
				"Cannot copy {} to {}",
				splash_image.display(),
				boot_path.display()
			)
		})?;

		let splash_mode = splash_mode.unwrap_or_default();
		write!(
			&mut self.inner,
			r#"insmod {ext}
if background_image --mode '{splash_mode}' "{boot_path}"/{target}; then
  set color_normal=white/black
  set color_highlight=black/white
else
  set menu_color_normal=cyan/blue
  set menu_color_highlight=white/blue
fi
"#,
			boot_path = self.grub_boot_path_normalized.display(),
			target = target.display(),
		)?;

		Ok(())
	}

	pub fn append_theme(&mut self) -> Result<()> {
		let Config {
			boot_path, theme, ..
		} = &self.config;
		let theme_dir = boot_path.join("theme");

		if theme_dir.exists() {
			fs::remove_dir_all(&theme_dir).with_context(|| {
				format!("Cannot clean up theme folder in {}", boot_path.display())
			})?;
		}

		let Some(theme) = theme else {
			return Ok(());
		};

		let mut modules_to_load = HashSet::new();
		let mut fonts = vec![];

		// TODO: Could be parallelized
		for entry in WalkDir::new(theme) {
			let entry = entry?;
			let relative = entry.path().strip_prefix(theme)?;

			if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
				match ext {
					"png" => _ = modules_to_load.insert("png"),
					"jpeg" | "jpg" => _ = modules_to_load.insert("jpeg"),
					"pf2" => fonts.push(relative.to_owned()),
					_ => {}
				}
			}

			fs::copy(entry.path(), theme_dir.join(relative))?;
		}

		for module in modules_to_load {
			write!(&mut self.inner, "insmod {module}")?;
		}

		write!(
			&mut self.inner,
			r#"# Sets theme.
set theme="{boot_path}"/theme/theme.txt
export theme
# Load theme fonts, if any
"#,
			boot_path = self.grub_boot_path_normalized.display(),
		)?;

		for font in fonts {
			write!(
				&mut self.inner,
				r#"loadfont "{}"/theme/{}"#,
				self.grub_boot_path_normalized.display(),
				font.display(),
			)?;
		}

		Ok(())
	}

	pub fn append_extra_config(&mut self) -> Result<()> {
		writeln!(&mut self.inner, "{}\n", self.config.extra_config)?;
		Ok(())
	}
}
