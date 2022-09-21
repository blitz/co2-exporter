{
  description = "CO2 monitor data exporter";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        craneLib = crane.lib.${system};
        src = ./.;

        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = [ pkgs.hidapi pkgs.libusb1 ];

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
        };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        co2-exporter = craneLib.buildPackage {
          inherit cargoArtifacts src nativeBuildInputs buildInputs;
        };
      in
        {
          checks = {
            # Build the crate as part of `nix flake check` for convenience
            inherit co2-exporter;

            # Run clippy (and deny all warnings) on the crate source,
            # again, resuing the dependency artifacts from above.
            #
            # Note that this is done as a separate derivation so that
            # we can block the CI if there are issues here, but not
            # prevent downstream consumers from building our crate by itself.
            co2-exporter-clippy = craneLib.cargoClippy {
              inherit cargoArtifacts src nativeBuildInputs buildInputs;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            };

            co2-exporter-doc = craneLib.cargoDoc {
              inherit cargoArtifacts src nativeBuildInputs buildInputs;
            };

            # Check formatting
            co2-exporter-fmt = craneLib.cargoFmt {
              inherit src;
            };

            # Audit dependencies
            co2-exporter-audit = craneLib.cargoAudit {
              inherit src advisory-db;
            };
          };

          packages.default = co2-exporter;

          apps.default = flake-utils.lib.mkApp {
            drv = co2-exporter;
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = builtins.attrValues self.checks;

            # Extra inputs can be added here
            nativeBuildInputs = with pkgs; [
              cargo
              rustc
            ];
          };
        });
}
