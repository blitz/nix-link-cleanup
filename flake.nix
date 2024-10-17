{
  description = "Nix Link Cleanup";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";

    systems.url = "github:nix-systems/default-linux";

    flake-parts.url = "github:hercules-ci/flake-parts";

    git-hooks-nix.url = "github:cachix/git-hooks.nix";
    git-hooks-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {

      imports = [
        inputs.git-hooks-nix.flakeModule
      ];

      systems = import inputs.systems;

      flake = { };

      perSystem = { config, system, pkgs, lib, ... }:
        let
          craneLib = inputs.crane.mkLib pkgs;

          # Common arguments can be set here to avoid repeating them later
          # Note: changes here will rebuild all dependency crates
          commonArgs = {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;
          };

          # Build *just* the cargo dependencies, so we can reuse
          # all of that work (e.g. via cachix) when running in CI
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          nix-link-cleanup = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        {
          pre-commit = {
            check.enable = true;

            settings = {
              src = ./.;
              hooks = {
                nixpkgs-fmt.enable = true;
                rustfmt.enable = true;
              };
            };
          };

          checks = {
            inherit nix-link-cleanup;

            # Run clippy (and deny all warnings) on the crate source,
            # again, reusing the dependency artifacts from above.
            #
            # Note that this is done as a separate derivation so that
            # we can block the CI if there are issues here, but not
            # prevent downstream consumers from building our crate by itself.
            clippy = craneLib.cargoClippy (commonArgs // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });
          };

          packages.default = nix-link-cleanup;

          apps.default = {
            type = "app";
            program = lib.getExe nix-link-cleanup;
          };

          devShells.default = craneLib.devShell {
            checks = config.checks;

            shellHook = config.pre-commit.installationScript;

            # Extra inputs can be added here; cargo and rustc are provided by default.
            packages = [
              # Nothing here yet.
            ];
          };
        };
    };
}
