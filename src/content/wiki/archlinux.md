---
title: Arch Linux
---

**Do not enter commands listed here verbatim without forethought**


## Installation guide

For future reference if I ever need to install this system again.

Reference:
- https://wiki.archlinux.org/title/installation_guide

### Prepare installation medium

Download iso from the [Arch Linux website](https://archlinux.org/download/).

Plug USB drive, usually it will be `/dev/sdb` or something similar. You can find it using this command:

```sh
sudo fdisk -l
```

Write iso file to the USB drive:

```sh
sudo dd bs=4M if=/path/to/iso/file of=/dev/sdX status=progress && sync
```

### Internet connection

Reference:
- https://wiki.archlinux.org/title/iwd

```sh
ping archlinux.org
```

If there's no connection check interfaces:
```sh
ip a
```

Use `iwctl` to connect to wifi:

```sh
iwctl
station [DEVICE] connect [SSID]
```

### Partitions

Reference:
- https://wiki.archlinux.org/title/partitioning
- https://wiki.archlinux.org/title/file_systems

```sh
cfdisk /dev/sda
```

| Partition    | Size | Type             |
| ------------ | ---- | ---------------- |
| 1            | 512M | EFI System       |
| 2 (optional) | 8G   | Linux swap       |
| 3            | rest | Linux filesystem |


Format partitions:

```sh
# EFI
mkfs.fat -F32 /dev/sda1
# Swap
mkswap /dev/sda2
swapon /dev/sda2
# Root filesystem
mkfs.ext4 /dev/sda3
```

### Install Arch

Reference:
- https://wiki.archlinux.org/title/installation_guide

```sh
pacman -Syy
mount /dev/sda3 /mnt
pacstrap -K /mnt base linux linux-firmware sudo vim dhcpcd
```

```sh
genfstab -U /mnt >> /mnt/etc/fstab
```

```sh
arch-chroot /mnt
```

```sh
pacman -S grub efibootmgr os-prober mtools
mkdir /boot/efi
mount /dev/sda1 /boot/efi
grub-install --target=x86_64-efi --bootloader-id=grub_uefi
grub-mkconfig -o /boot/grub/grub.cfg
```

### Desktop environment

Reference:
- https://wiki.archlinux.org/title/SDDM
- https://wiki.archlinux.org/title/KDE

Install KDE:

```sh
pacman -S xorg-server xorg-apps
pacman -S nvidia nvidia-utils
pacman -S sddm plasma networkmanager plasma-nm
```

Enable systemd services for the DE:

```sh
systemctl enable dhcpcd
systemctl enable sddm
```


## Fcitx5 + Mozc

Reference:
- https://wiki.archlinux.org/title/Fcitx5
- https://wiki.archlinux.org/title/Mozc

```sh
sudo pacman -S fcitx5-im fcitx5-mozc
fcitx5-config-qt
```

In `/etc/environment` add:

```sh
GTK_IM_MODULE=fcitx
QT_IM_MODULE=fcitx
XMODIFIERS=@im=fcitx
SDL_IM_MODULE=fcitx
GLFW_IM_MODULE=ibus
```

### KDE Plasma 6

> Detect GTK_IM_MODULE and QT_IM_MODULE being set and Wayland Input method
> frontend is working. It is recommended to unset GTK_IM_MODULE and
> QT_IM_MODULE and use Wayland input method frontend instead. For more details
> see https://fcitx-im.org/wiki/Using_Fcitx_5_on_Wayland#KDE_Plasma 

