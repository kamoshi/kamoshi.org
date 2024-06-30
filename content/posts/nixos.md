---
title: My thoughts on NixOS
date: 2024-06-30T20:57:37Z
desc: >
    A few weeks ago I decided to try out NixOS. 
---

For about a year I have used Arch Linux, however a few weeks ago I decided to
try out [NixOS](https://nixos.org/). Here are some of my thoughts about it.

# Configuration

Overall, NixOS is a very nice distribution which feels much unlike anything
else you can find out there. In particular, the way you configure NixOS is
different from what you would do on Arch.

The way you configure NixOS is by writing the configuration in a language
called Nix. This configuration then is read by the Nix daemon which instructs
the Nix package manager to install the required packages in the Nix store.

For example if you wanted to enable OpenGL in your system you would have to
write something along the lines of

```nix
hardware.graphics = {
	enable = true;
	enable32Bit = true;
};
```

Nix would install every single thing require to enable it completely
automatically.

I found this brilliant, because it essentially means that someone out there has
prepared the optimal config and now everyone gets to reuse it completely for
free.

Take [Steam](https://nixos.wiki/wiki/Steam) for example, if you want to play
games you gave to write

```nix
programs.steam.enable = true;
```

This single line not only installs Steam on your system, but also provides
configuration which makes your system more tailored for running any games.

Let's say you are running a server and want to enable
[Nginx](https://nixos.wiki/wiki/Nginx). You can do it like this to get the
optimal configuration for free, without having to deal with Nginx config files.

```nix
services.nginx = {
	enable = true;
	recommendedGzipSettings = true;
	recommendedOptimisation = true;
	recommendedProxySettings = true;
	recommendedTlsSettings = true;
};
```

This is amazing.

The way NixOS works makes it extremely easy to not only configure the system,
but also save the configuration, because it's trivial to commit it into a VCS.


# Home

However, there are some rough edges.

By default there isn't really a way to configure the home directory for a
single user. The thing you would traditionally call dotfiles isn't really
covered by vanilla NixOS.

Instead you have to install a third party module called
[home-manager](https://nix-community.github.io/home-manager/). This allows you
to install software for a single user, as well as populate the home directory
with various configuration files.

I have decided to use this module in my configuration, and generally it was
quite nice to use, but there were some rough edges. For example, when I tried
to install KDE Connect via home-manager, it would install the one for Plasma 5
instead of Plasma 6.

```nix
services.kdeconnect.enable = true;
```

I found that it's better to just install this program in the main config instead.


# Channels

NixOS has different channels you can use for the packages. A channels is
something like a current set of packages and/or options which can be used in
your config. Currently you can for example use the `24.05` channel for stable,
or `unstable` for unstable (which actually is quite stable).

I'm kind of on the fence about channels.

On one hand it's nice that the set of packages in `24.05` is nearly set in
stone, because then updating your system is less resource intensive. On the
other hand, the packages aren't updated much which means you're stuck with
Neovim 0.9.5 for the next half a year or so -- sad!

So if you want to have newer Neovim version you have to pull it from
`unstable`. Yes, it's possible to do it while having your system pinned to
stable channel. But at that point isn't it better to just pull everything from
unstable?

This is what I went with and it works for me so far, but I'm not sure how it
will be like in the long run...


# Flakes

There is apparently a completely different and experimental (unstable) approach
to versioning the system, which breaks with the old channel system. I haven't
used it myself though, so I can't give an opinion about it specifically.

One thing I can say though is that I find it pretty confusing how there are
several competing systems for writing the system configuration, out of which
one is experimental, but lots of people decided to use it anyways...


# Final thoughts

NixOS... is interesting. It's different, you do things differently here, even
though under all of these layers of abstraction, there's still Linux.

It gets some things extremely right.

It lets you pull many versions of the same package, because everything is
hashed and kept in a flat store on your hard drive.

It lets you easily write the dependencies for your projects, because you can
just write a `shell.nix` file which specifies them and then use `nix-shell`
command to pull them all into your shell.

For example to build this website I have a `shell.nix` file like this

```nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
	buildInputs = with pkgs; [
		cargo
		clippy
		esbuild
		nodePackages.pnpm
		pagefind
		python3
		rust-analyzer
		rustc
		rustfmt
	];

	RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
```

I can just do this to build it

```
kamov@msi ~/D/website (main)> nix-shell --command fish
kamov@msi ~/D/website (main)> make
```

Everything is isolated, you can easily rollback your system to a previous state.

You can experiment.

You can have lots of fun!

But it feels rough around the edges.

If you feel like having an adventure go right ahead...

Cheers!
