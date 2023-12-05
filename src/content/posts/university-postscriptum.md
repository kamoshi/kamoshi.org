---
title: University Postscriptum
date: 2023-07-25T13:29:44.842Z
---

Now that I am free to do literally anything with my free time, I have decided to go all in and learn all these cool things that are supposedly hard to use.


## NixOS

I started off by migrating my server to [NixOS](https://nixos.org/), which is a GNU/Linux distribution based on the Nix package manager. It is quite... exotic. In this distribution all configuration is written using the Nix language, which is a Turing complete purely functional programming language, in some aspects similar to Haskell.

Here's an example of how you would use Nix to enable Podman virtualization in the system:

```nix
{ config, pkgs, ... }:
{
  virtualisation = {
    podman = {
      enable = true;
      dockerCompat = true;
    };
}
```

I have found NixOS to be much better than classic DevOps tools such as Ansible for managing the state of machines, especially as a singular person and not a team of SysOps guys. It lets you specify the entire state of the server (excluding the secrets) in a declarative way. There is little chance of failure, and if something does fail, you can just roll back the entire state to the last working configuration. I've uploaded the configs I use to a [GitHub repository](https://github.com/kamoshi/server/blob/main/hosts/kamoshi/configuration.nix) for anyone curious.

It was a pretty tough ride the first time I tried to use this distribution, but once you get the ball rolling everything gets easier. And since it is fully declarative I don't really have to do any maintenance of the server, it's nearly fully autonomous.


## Neovim

I've decided to ditch fully-featured IDEs such as IntelliJ and lighter editors such as VS Code, instead I use Neovim. Neovim is really nice, but it's a modal, terminal based editor, which does take some time to get used to. I use it partially because all the cool guys use Neovim or Emacs, but also because I like it a lot! It has a lot of useful keybinding, it's very fast, very light, it takes nearly no resources to run.

I also like the fact that the entire config for Neovim can be stored in a git repository. I uploaded mine to a [GitHub repository](https://github.com/kamoshi/dotfiles/tree/main/nvim) for anyone curious. You can use Lua to configure this editor instead of Vimscript, which is pretty cool, I like Lua.

![Neovim screengrab](@assets/posts/neovim.png)


## Arch Linux

Btw, I moved from Kubuntu to Arch Linux. Kubuntu is a really sweet distribution which I would wholeheartedly recommend to anyone dipping their toes in the GNU/Linux world. It's very beginner-friendly like Ubuntu, but also features KDE Plasma and KDE apps. KDE is in my opinion the best desktop environment, it is also similar to Windows, which means it will be pretty intuitive for anyone coming from that world.

People say that installing Arch is hard, but honestly it really isn't. It just doesn't have a graphical installer that will do it all for you, and you also need to know how to use the command line interface. Some people don't know how to use terminals, if you're that person you shouldn't be using GNU/Linux in the first place... or you should start using it right now, because you're missing out!!!

I now know why people like Arch so much.

It is free of bloat. You aren't bombarded with packages you will never use and package managers which are pure overhead (looking at you Snap!). When you install Arch you only ever install what you really need. Personally - I don't even have any office suite installed, the only text editor I have right now is Neovim. And if I ever need more I will use `pacman`, it is that easy.

The package repository for Arch Linux is really huge. When I used Kubuntu, there were things I had to install by downloading `.tar.gz` files or `.deb` packages. This contributed to the fact that the operating system was harder and harder to manage and keep up to date. With Arch everything I need is in the official package repository, or at least in the unofficial Arch User Repository. Downloading every program via your package manager is not only clean and convenient, but also feels like it's how GNU/Linux was meant to be used in the first place.

On top of all that, Arch Linux is a rolling release distro, which means that everything I use is up-to-date. Right now the latest released Neovim version is 9.1, and my Neovim, which I installed with `pacman`, has version 9.1. When I used Kubuntu, the Neovim I had there had version 0.7.2, which was released (at the time of writing) a year ago. When you use Neovim installed via `apt` you can't use many plugins, because they are incompatible, so you end up downloading `.tar.gz` and using that instead. But then you miss out on all the good things your system's package manager provides. With Arch you get to use both package manager and up-to-date packages :heart:

Below is an obligatory neofetch screenshot.

![Obligatory neofetch screengrab](@assets/posts/arch.png)


For some time I probably won't be writing anything new in here. It's really ironic, but now that I do have a lot of time, I don't really have anything interesting to write about. I have a ton of things to do, which I couldn't really do while I was studying full time at a university.

So thanks and until next time! :wave:

