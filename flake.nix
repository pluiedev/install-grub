{
  inputs.nixpkgs.url = "nixpkgs";

  outputs =
    { self, nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;
      systems = [ "x86_64-linux" ];
      perSystem = f: lib.genAttrs systems (s: f nixpkgs.legacyPackages.${s});
    in
    {
      devShells = perSystem (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            nixfmt-rfc-style
            self.packages.${pkgs.system}.default
          ];
        };
      });

      packages = perSystem (pkgs: {
        default = pkgs.callPackage ./package.nix { };
      });
    };
}
