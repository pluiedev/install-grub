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
        default = pkgs.mkShell { nativeBuildInputs = with pkgs; [ nixfmt-rfc-style ]; };
      });
    };
}
