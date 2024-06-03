{
  lib,
  rustPlatform,
  btrfs-progs,
  util-linux,
}:
rustPlatform.buildRustPackage {
  pname = "install-grub";
  version = "0.1.0";

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.lock
      ./Cargo.toml
      ./src
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  BLKID = lib.getExe' util-linux "blkid";
  BTRFS = lib.getExe' btrfs-progs "btrfs";
  DISTRO_NAME = "NixOS";
}
