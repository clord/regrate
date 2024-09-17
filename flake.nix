{
  description = "Regrate helps with migrations; especially in an environment with concurrent updates.";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = {
        # self',
        # pkgsm,
        lib,
        system,
        ...
      }: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        rustBuild = rustPlatform.buildRustPackage {
          pname = "regrate";
          noCheck = true;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = [
            pkgs.darwin.apple_sdk.frameworks.Security
          ];
          buildFeatures = [];
        };
      in {
        packages = rec {
          regrate = rustBuild;
          default = regrate;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            nixfmt
            rustVersion
          ];
        };

        apps = {
          info = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "info" ''
              echo "HEY WORLD"
            '';
          };
        };
      };
    };
}
