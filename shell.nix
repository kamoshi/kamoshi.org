{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rust-analyzer
    rustfmt
    clippy
    cargo
  ];
}
