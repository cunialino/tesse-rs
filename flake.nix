{
  description = "Example flake: dev shell with fenix-provided Rust (fixed)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      fenix,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        fnx = fenix.packages.${system};

        profileName = "stable";
        chosen = fnx.${profileName};

        fenixToolchain = chosen.toolchain;
      in
      {
        devShells.default = pkgs.mkShell {
          name = "rust-dev-with-fenix";

          buildInputs = [
            fenixToolchain
          ];

          shellHook = ''
            echo "fenix Rust toolchain profile: ${profileName}"
            echo "Toolchain runtime check:"
            rustc --version || echo "(rustc not available yet)"
            cargo --version || echo "(cargo not available yet)"
            rust-analyzer --version || echo "(rust-analyzer not available yet)"
          '';
        };
      }
    );
}
