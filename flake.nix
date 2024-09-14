{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-compat = {
      url = "github:edolstra/flake-compat/master";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      ...
    }:
    let
      eachSystem =
        f:
        nixpkgs.lib.genAttrs
          [
            "aarch64-darwin"
            "aarch64-linux"
            "x86_64-darwin"
            "x86_64-linux"
          ]
          (
            system:
            let
              pkgs = import nixpkgs {
                inherit system;
                config = { };
                overlays = [ (import rust-overlay) ];
              };
            in
            f {
              inherit pkgs;
              craneLib = (crane.mkLib pkgs).overrideToolchain (
                p:
                p.rust-bin.stable.latest.default.override {
                  extensions = [ "rust-analyzer" ];
                }
              );
            }
          );
    in
    {
      formatter = eachSystem (pkgs: pkgs.nixfmt-rfc-style);
      packages = eachSystem (
        { pkgs, craneLib }:
        {
          default = craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;
            buildInputs = [ ];
          };
        }
      );
      devShells = eachSystem (
        { pkgs, craneLib }:
        {
          default = craneLib.devShell {
            inputsFrom = [ self.packages.${pkgs.system}.default ];
            packages = with pkgs; [
              cargo-expand
              cargo-watch
              evcxr
              mold
              sccache
            ];
            CARGO_INCREMENTAL = "0";
            RUSTC_WRAPPER = "sccache";
            RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          };
        }
      );
    };
}
