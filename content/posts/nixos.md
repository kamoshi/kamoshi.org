---
title: My thoughts on NixOS
date: 2024-06-30T20:57:37Z
tags: [nix, nixos]
desc: >
  A few weeks ago I decided to try out NixOS.
---

For about a year, I have used Arch Linux. However, a few weeks ago, I decided
to try out [NixOS](https://nixos.org/). Here are some of my thoughts about it.

## Configuration

Overall, NixOS is a very unique distribution that feels unlike anything else
out there. In particular, the way you configure NixOS is different from what
you would do on Arch Linux.

### The Nix language

The way you configure NixOS is by writing the configuration in a declarative
language called Nix. This configuration is then read by the Nix daemon, which
instructs the Nix package manager to install the required packages in the Nix
store.

For example, if you wanted to enable OpenGL, you would have to write something
along the lines of:

```nix
hardware.graphics = {
	enable = true;
	enable32Bit = true;
};
```

NixOS would install every single thing required to enable OpenGL completely
automatically.

I found this approach brilliant because it essentially means that someone out
there has prepared the optimal configuration, and now everyone gets to reuse it
completely for free.

Take [Steam](https://nixos.wiki/wiki/Steam), for example. If you want to play
games, you need to write:

```nix
programs.steam.enable = true;
```

This single line not only installs Steam on your system but also provides
configurations that make your system more tailored for running games.

### Configuring services

Let's say you are running a server and want to enable
[Nginx](https://nixos.wiki/wiki/Nginx). You can do it like this to get the
optimal configuration for free, without having to deal with Nginx config files:

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

The way NixOS works makes it extremely easy not only to configure the system
but also to save the configuration. It's trivial to commit it into a version
control system (VCS), ensuring reproducibility and ease of deployment across
multiple systems.

## Home configuration

However, there are some rough edges.

By default, there isn't really a way to configure the home directory for a
single user. The traditional concept of "dotfiles" isn't really covered by
vanilla NixOS.

Instead, you have to install a third-party module called
[home-manager](https://nix-community.github.io/home-manager/). This allows you
to install software for a single user, as well as populate the home directory
with various configuration files.

I decided to use this module in my configuration, and generally, it was quite
nice to use, but there were some rough edges. For example, when I tried to
install KDE Connect via home-manager, it installed the version for Plasma 5
instead of Plasma 6.

```nix
services.kdeconnect.enable = true;
```

I found that it's better to just install this program in the main config
instead. Other than this home-manager works pretty good, and it is very useful,
so I recommend using it.

## Channels

NixOS has different channels you can use for packages. A channel is essentially
a current set of packages and/or options that can be used in your
configuration. Currently, you can use the `24.05` channel for stable, or
`unstable` for the latest updates (which is actually quite stable).

I'm kind of on the fence about channels.

On one hand, it's nice that the set of packages in `24.05` is nearly set in
stone because updating your system is less resource-intensive. On the other
hand, the packages aren't updated frequently, which means you're stuck with
Neovim 0.9.5 for the next half a year or so -- sad!

So, if you want to have a newer Neovim version, you have to pull it from
`unstable`. Yes, it's possible to do this while keeping your system pinned to
the stable channel. But at that point, isn't it better to just pull everything
from `unstable`?

This is what I went with, and it works for me so far, but I'm not sure how it
will be in the long run...

## Flakes

There is a completely different and experimental (unstable) approach to
versioning the system, called flakes, which breaks with the old channel system.
I haven't used it myself, so I can't give an opinion about it specifically.

One thing I can say, though, is that I find it pretty confusing how there are
several competing systems for writing the system configuration, out of which
one is experimental, but lots of people decided to use it anyways.

# Final thoughts

NixOS... is interesting. It's different, you do things differently here, even
though under all these layers of abstraction, it's still Linux.

It gets some things extremely right.

It lets you pull many versions of the same package because everything is hashed
and kept in a flat store on your hard drive.

It lets you easily write the dependencies for your projects because you can
just write a `shell.nix` file specifying them and then use the `nix-shell`
command to pull them all into your shell.

For example, to build this website, I have a `shell.nix` file like this:

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

I can just do this to build it:

```
kamov@msi ~/D/website (main)> nix-shell --command fish
kamov@msi ~/D/website (main)> make
```

Everything is isolated, you can easily roll-back your system to a previous state.

You can experiment.

You can have lots of fun!

But it feels rough around the edges.

If you feel like having an adventure, go right ahead...

Cheers!
