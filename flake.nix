{
  description = "A devShell example";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        used = with pkgs; [
          rust-bin.stable.latest.default
          cargo
          openssl
          git
          deno
          # biome
          esbuild
        ];
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = used;

          shellHook = ''
            echo ""
            echo "Available tools:"
            echo "  - Rust: $(rustc --version)"
            echo "  - Cargo: $(cargo --version)"
            echo "  - Deno: $(deno --version | head -n1)"
            # echo "  - Biome: $(biome --version | head -n1)"
            echo "  - Esbuild: $(esbuild --version | head -n1)"

            echo ""
            echo "Run 'make' to see available build targets"
          '';

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      }
    );
}
