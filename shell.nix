{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
	buildInputs = with pkgs; [
		cargo
		clippy
		rustfmt
		rust-analyzer
		pagefind
		esbuild
		nodePackages.pnpm
		python3
	];
}
