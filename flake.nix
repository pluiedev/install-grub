{
  inputs.nixpkgs.url = "nixpkgs";

  outputs =
    { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;
      systems = [ "x86_64-linux" ];
      perSystem = f: lib.genAttrs systems (s: f nixpkgs.legacyPackages.${s});
    in
    {
      devShells = perSystem (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ nixfmt-rfc-style ];
          buildInputs = with pkgs; [
            btrfs-progs
            util-linux
          ];

          BLKID = pkgs.lib.getExe' pkgs.util-linux "blkid";
          BTRFS = pkgs.lib.getExe' pkgs.btrfs-progs "btrfs";
          DISTRO_NAME = "NixOS";
        };
      });
    };
}
