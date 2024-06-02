use std::path::PathBuf;

use anyhow::{bail, Result};
use libxml::{parser::Parser, xpath::Context};

use crate::grub::FsIdentifier;

macro_rules! config {
  ($($field:ident : $ty:ty => $key:ident),*$(,)?) => {
    pub struct Config {
      $(
        pub $field: $ty
      ),*
    }

    impl Config {
      pub fn new(filename: &str) -> Result<Self> {
		    let parser = Parser::default();
		    let document = parser.parse_file(filename)?;
		    let Ok(ctx) = Context::new(&document) else {
		    	bail!("Failed to create XPATH context")
		    };

		    Ok(Self {$(
          $field: <$ty as ConfigValue>::read(&ctx, stringify!($key))?
        ),*})
      }
    }
  }
}

config! {
  grub: Option<PathBuf> => grub,
  grub_target: Option<PathBuf> => grubTarget,
  grub_efi: Option<PathBuf> => grubEfi,
  grub_target_efi: Option<PathBuf> => grubTargetEfi,

  extra_config: String => extraConfig,
  extra_prepare_config: String => extraPrepareConfig,
  extra_per_entry_config: Option<String> => extraPerEntryConfig,
  extra_entries: String => extraEntries,
  extra_entries_before_nixos: bool => extraEntriesBeforeNixOS,

  splash_image: Option<PathBuf> => splashImage,
  splash_mode: Option<String> => splashMode,
  background_color: Option<String> => backgroundColor,

  entry_options: String => entryOptions,
  sub_entry_options: String => subEntryOptions,

  configuration_limit: usize => configurationLimit,
  copy_kernels: bool => copyKernels,

  timeout: u32 => timeout,
  timeout_style: String => timeoutStyle,

  default_entry: String => default,
  fs_identifier: FsIdentifier => fsIdentifier,

  boot_path: PathBuf => bootPath,
  store_path: PathBuf => storePath,

  gfx_mode_efi: String => gfxmodeEfi,
  gfx_mode_bios: String => gfxmodeBios,
  gfx_payload_efi: String => gfxpayloadEfi,
  gfx_payload_bios: String => gfxpayloadBios,

  font: PathBuf => font,
  theme: Option<PathBuf> => theme,
  shell: PathBuf => shell,
  path: String => path,

  users: Vec<User> => users,

  use_os_prober: bool => useOSProber,

  can_touch_efi_variables: bool => canTouchEfiVariables,
  efi_install_as_removable: bool => efiInstallAsRemovable,
  efi_sys_mount_point: PathBuf => efiSysMountPoint,

  bootloader_id: String => bootloaderId,
  force_install: bool => forceInstall,

  devices: Vec<PathBuf> => devices,
  extra_grub_install_args: Vec<String> => extraGrubInstallArgs,
  full_name: String => fullName,
  full_version: String => fullVersion,
}

impl Config {
	pub fn save_default(&self) -> bool {
		self.default_entry == "saved"
	}
}

pub trait ConfigValue: Sized {
	fn read(ctx: &Context, name: &str) -> Result<Self>;
}

impl<T: ConfigValue> ConfigValue for Option<T> {
	fn read(ctx: &Context, name: &str) -> Result<Self> {
		Ok(if let Ok(v) = <T as ConfigValue>::read(ctx, name) {
			Some(v)
		} else {
			None
		})
	}
}
impl ConfigValue for String {
	fn read(ctx: &Context, name: &str) -> Result<Self> {
		let xpath = format!("/expr/attrs/attr[@name = '{name}']/*/@value");

		let Ok(value) = ctx.evaluate(&xpath) else {
			bail!("Could not find string with key {name} in XML document")
		};
		Ok(value.to_string())
	}
}
impl ConfigValue for Vec<String> {
	fn read(ctx: &Context, name: &str) -> Result<Self> {
		let xpath = format!("/expr/attrs/attr[@name = '{name}']/list/string/@value");

		// I'm not sure why `findvalues` takes `&mut self`...?
		let Ok(values) = ctx.evaluate(&xpath) else {
			bail!("Could not find list with key {name} in XML document")
		};
		Ok(values.get_nodes_as_str())
	}
}
impl ConfigValue for bool {
	fn read(ctx: &Context, name: &str) -> Result<Self> {
		Ok(<String as ConfigValue>::read(ctx, name)? == "true")
	}
}
impl ConfigValue for PathBuf {
	fn read(ctx: &Context, name: &str) -> Result<Self> {
		Ok(PathBuf::from(<String as ConfigValue>::read(ctx, name)?))
	}
}
impl ConfigValue for Vec<PathBuf> {
	fn read(ctx: &Context, name: &str) -> Result<Self> {
		Ok(<Vec<String> as ConfigValue>::read(ctx, name)?
			.into_iter()
			.map(PathBuf::from)
			.collect())
	}
}
impl ConfigValue for Vec<User> {
	fn read(ctx: &Context, name: &str) -> Result<Self> {
		let hashed_password_file_key =
			r#"./attrs/attr[@name = "hashedPasswordFile"]/string/@value"#;
		let hashed_password_key = r#"./attrs/attr[@name = "hashedPassword"]/string/@value"#;
		let password_file_key = r#"./attrs/attr[@name = "passwordFile"]/string/@value"#;
		let password_key = r#"./attrs/attr[@name = "password"]/string/@value"#;

		let users_key = format!(r#"/expr/attrs/attr[@name = "{name}"]/attrs/attr"#);
		let Ok(nodes) = ctx.evaluate(&users_key) else {
			bail!("Could not find users in XML document")
		};

		nodes
			.get_nodes_as_vec()
			.into_iter()
			.map(|user| {
				let get_value = |xpath: &str| -> Result<String, ()> {
					ctx.node_evaluate(xpath, &user).map(|s| s.to_string())
				};

				let Ok(name) = get_value("@name") else {
					bail!("Name not found for user")
				};

				let hashed_password = if let Ok(f) = get_value(hashed_password_file_key) {
					Some(std::fs::read_to_string(f)?)
				} else if let Ok(p) = get_value(hashed_password_key) {
					Some(p)
				} else {
					None
				};

				let password = if let Some(p) = hashed_password {
					if p.starts_with("grub.pbkdf2.") {
						Password::Hashed(p)
					} else {
						bail!("Password hash for GRUB user '{name}' is not valid!")
					}
				} else {
					let pass = if let Ok(f) = get_value(password_file_key) {
						std::fs::read_to_string(f)?
					} else if let Ok(p) = get_value(password_key) {
						p
					} else {
						bail!("GRUB user '{name}' has no password!")
					};

					Password::Plain(pass)
				};

				Ok(User { name, password })
			})
			.collect::<Result<Vec<_>, _>>()
	}
}

macro_rules! int_impl {
  ($($ty:ty)*) => {
    $(
      impl ConfigValue for $ty {
      	fn read(ctx: &Context, name: &str) -> Result<Self> {
      		Ok(<String as ConfigValue>::read(ctx, name)?.parse::<Self>()?)
      	}
      }
    )*
  }
}
int_impl!(u8 u16 u32 u64 u128 usize);

pub struct User {
	pub name: String,
	pub password: Password,
}

pub enum Password {
	Plain(String),
	Hashed(String),
}
