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
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            rust-bin.beta.latest.default
            deno
            pkg-config
            openssl
            git
          ];

          shellHook = ''
            echo ""
            echo "Available tools:"
            echo "  - Rust: $(rustc --version)"
            echo "  - Deno: $(deno --version | head -n1)"
            echo ""
            echo "Run 'make' to see available build targets"
          '';
        };
      }
    );
}
