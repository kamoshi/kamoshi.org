---
title: "Password Managers"
date: 2023-01-18T18:29:58+01:00
---

# Password managers
Maciej Jur  
2023.01.20

-----

## Password managers
- Introduction
- Types of password managers
- Choosing a password manager
- Using a password manager
- LastPass leak
- Summary

-----

## Not every password is safe
![Password](/static/slides/password-managers/safe-password.png)

---

Password I used for most of my accounts in the past:
![My password](/static/slides/password-managers/my-password.png)
www.security.org/how-secure-is-my-password/

-----

### Why reuse passwords?
![a](/static/slides/password-managers/threatlist-2.png)
https://www.darkreading.com/endpoint/password-reuse-problems-persist-despite-known-risks

---

![a](/static/slides/password-managers/threatlist-1.png)
https://www.darkreading.com/endpoint/password-reuse-problems-persist-despite-known-risks

-----

### Some more statistics
![a](/static/slides/password-managers/2021-09-23-image.jpg)
https://www.techspot.com/news/91388-most-people-reuse-passwords-across-multiple-sites.html

---

![a](/static/slides/password-managers/2021-09-23-image-2.webp)
https://www.techspot.com/news/91388-most-people-reuse-passwords-across-multiple-sites.html

---

![a](/static/slides/password-managers/2021-09-23-image-3.webp)
https://www.techspot.com/news/91388-most-people-reuse-passwords-across-multiple-sites.html

-----

### Password managers can help
![a](/static/slides/password-managers/vault.png)

-----

## Types of password managers
- Offline
- Online
- Stateless
- Hardware

-----

### Offline password managers

---

#### Example: KeePass(XC)
![KeePassXC](/static/slides/password-managers/keepassxc.webp)

---

#### These password managers can still be online
Dropbox, Google Drive, SyncThing, etc.
![replication](/static/slides/password-managers/replication.png)
https://keepass.info/help/kb/trigger_examples.html

---

#### Pros
- Simple and safe to use
- Your password vault never leaves your device (unless you want it to)
- You can transfer vaults between devices using thumb drives or cloud sync

---

#### Cons
- You have to move/sync the vault beetwen devices on your own
- That could be a hassle

-----

### Online password managers

---

#### Example: LastPass
![lastpass](/static/slides/password-managers/lastpass.png)

---

#### Pros
- Most of the pros of offline password managers
- Automatic sync, can access your vault as long as you have access to the Internet
- It's slightly more convenient

---

#### Cons
- The vault lives on some random server
- You have to trust the ___service provider___
- You have to trust the ___security___ of the service provider

-----

### Stateless password managers

Instead of saving your passwords and encrypting them with a key derived from a master password, these password managers generate passwords on the fly by hashing a master password with the website name.

---

#### Example: LessPass
![a](/static/slides/password-managers/lesspass-graph.png)

---

#### Pros
- You don't have to synchronize your vault between any of your devices.

---

#### Cons
- If your master password is compromised, all of your passwords are.
- If a website has a password policy, you might not be able to generate a password that respects it.
- If password needs to be updated for whatever reason, you need to keep that state somewhere.
  Example: Password for "StackOverflow2"
- If you already have some passwords that you can't change (for various reasons), a static password generator won't help you.

-----

### Hardware password managers

---

#### Example: OnlyKey
It emulates a HID keyboard and can be programmed to navigate the steps to log in to pretty much any website, even if the login requires tabbing around multiple screens.

---

<iframe width="1024" height="576" src="https://www.youtube.com/embed/CBDKx2_br3g" title="How-To: Secure your Workstation and Online Accounts with OnlyKey" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>

---

#### Pros
- Pin protected
- Durable, waterproof, and tamper resistant design
- The device isn't connected to the Internet

---

#### Cons
- Cost (260,00 PLN) and learning curve
- There's a limit to how much you can store
- OnlyKey can store up to 24 online accounts

-----

## Choosing a password manager
- Type of password manager - this is the easier choice
- Which password manager - this is the harder choice

---

Spoiler alert:  
`There are a lot of them`

---

Some of them:
![Various password managers](/static/slides/password-managers/various-password-managers.png)
https://allthatsaas.com/roundup/best-password-managers/

---

Do I use any one of these?

---

Answer: Yes  
![I use bitwarden](/static/slides/password-managers/i-use-bitwarden.png)

-----

### A short comparison
![comparison](/static/slides/password-managers/comparison1.jpg)
https://blog.devolutions.net/2019/01/updated-2019-most-popular-password-managers-compared/

---

#### Why do I use Bitwarden?
Honestly, I'm not sure.  

But it has sync and I like the fact that its components are open-source:  
https://github.com/bitwarden

-----

## Using a password manager
I will show Bitwarden, because I already know it.
![Bitwarden-example](/static/slides/password-managers/bitwarden-sample.webp)

-----

### Vault
![bitwarden vault](/static/slides/password-managers/bitwarden-vault.png)

---

#### Mobile vault
![bitwarden vault mobile](/static/slides/password-managers/bitwarden-vault-mobile.png)

-----

### Autofill
![bitwarden autofill](/static/slides/password-managers/bitwarden-autofill.png)

---

#### Mobile autofill
![bitwarden autofill mobile](/static/slides/password-managers/bitwarden-autofill-mobile.jpg)

-----

### Generating passwords
![bitwarden generating](/static/slides/password-managers/bitwarden-generating.png)

-----

### Out of curiosity
How do you configure OnlyKey?

---

#### Slots
![onlykey-slots](/static/slides/password-managers/onlykey-slots.png)
https://docs.onlykey.io/usersguide.html

---

#### Autofill
![onlykey-autofill](/static/slides/password-managers/onlykey-autofill.png)
https://docs.onlykey.io/usersguide.html

---

#### Even more complex
You need to perform the following:
1. Enter the Username
2. Press TAB
3. Press RETURN
4. Wait for website to load next page
5. Enter the password
6. Press TAB
7. Press RETURN

---

You can enter `\t` or `\r` inline with slot data to type the extra TAB or RETURN and `\d3` to DELAY 3 seconds.

Username:  
`onlykey \t  \r  \d3 `

Password:  
`password \t  \r `

---

![onlykey-advanced-autofill](/static/slides/password-managers/onlykey-advanced-autofill.png)
https://docs.onlykey.io/usersguide.html

-----

## LastPass Leak

December 22, 2022

https://blog.lastpass.com/2022/12/notice-of-recent-security-incident/

---

> "Based on our investigation to date, we have learned that an unknown threat actor accessed a cloud-based storage environment ___leveraging information obtained from the incident we previously disclosed in August of 2022___."

---

### August 2022
> "An employee’s work account was compromised to gain unauthorized access to the company’s development environment, which stores some of LastPass’ source code."

https://techcrunch.com/2022/12/14/parsing-lastpass-august-data-breach-notice/

---

> "The threat actor was also able to copy a backup of customer vault data from the encrypted storage container which is stored in a proprietary binary format that contains both unencrypted data, such as website URLs, as well as fully-encrypted sensitive fields such as website usernames and passwords, secure notes, and form-filled data."

---

> "These encrypted fields remain secured with 256-bit AES encryption and can only be decrypted with a unique encryption key derived from each user’s master password using our Zero Knowledge architecture. As a reminder, the master password is never known to LastPass and is not stored or maintained by LastPass."

---

### What do we get from that?
- Password managers increase our security...
- ...but not ultimately

---

### It's not the end of the world
Thanks to zero knowledge architecture the attacker still has to crack the master password, which could take years ...

---

... as long as we used a secure password for the master password.

-----

## Summary
- Password manager helps manage passwords
- Allows us to use unique, complex passwords for different accounts without having to remember them all
- By using a password manager, you can improve your security without sacrificing convenience.
- To choose the best password manager for your needs, consider factors such as security, compatibility and convenience.

---

In general, password managers are an essential tool for anyone who wants to improve their online security and protect their personal information.

-----

## Any questions?
