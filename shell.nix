{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
	buildInputs = with pkgs; [
		cargo
		clippy
		esbuild
		nodejs
		nodePackages.pnpm
		pagefind
		python3
		rust-analyzer
		rustc
		rustfmt
	];

	RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
