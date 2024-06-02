{
  inputs.nixpkgs.url = "nixpkgs";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" ];
      perSystem = f: nixpkgs.lib.genAttrs systems (s: f nixpkgs.legacyPackages.${s});
    in
    {
      devShells = perSystem (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ pkg-config rustc cargo ];
          buildInputs = with pkgs; [ libxml2 ];
        };
      });
    };
}
