{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (localSystem:
      let
        pkgs = import nixpkgs {
          inherit localSystem;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;

          nativeBuildInputs = with pkgs; [ cargo-auditable pkg-config ];
        };

        cargoArtifacts =
          craneLib.buildDepsOnly (commonArgs // { pname = "deps"; });

        cargoClippy = craneLib.cargoClippy (commonArgs // {
          inherit cargoArtifacts;
          pname = "clippy";
        });

        cargoDoc = craneLib.cargoDoc (commonArgs // {
          inherit cargoArtifacts;
          pname = "doc";
        });

        foo-bar = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "foo-bar";
        });
      in {
        checks = { inherit foo-bar cargoClippy cargoDoc; };

        apps = rec {
          foo = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "foo" ''
              ${self.packages.${localSystem}.foo}/bin/foo-bar
            '';
          };

          default = foo;
        };

        packages = rec {
          foo = foo-bar;

          default = pkgs.symlinkJoin {
            name = "all";
            paths = [ foo ];
          };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${localSystem};
          packages = with pkgs; [ cargo-watch rust-analyzer ];
        };
      });
}
