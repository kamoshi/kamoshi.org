---
title: Firefox
---

I use Firefox as my primary web browser, which allows me to easily sync my data
and tabs between Linux and MacOS.

A few extensions that I personally find useful:

* [Consent-O-Matic][consents]: Automatic handling of GDPR consent forms
* [Flagfox][flagfox]: Displays a flag depicting the location of the current
  server
* [SteamDB][steamdb]: Adds SteamDB links and new features on the Steam store and
  community.
* [Tampermonkey][tamper]: Change the web at will with userscripts
* [uBlock Origin][ublock]: Finally, an efficient blocker. Easy on CPU and
  memory.
* [Unhook - Remove YouTube Recommended & Shorts][unhook]: Hide YouTube related
  videos, shorts, comments, suggestions wall, homepage recommendations...
* [Yomitan Popup Dictionary][yomitan]: Popup dictionary for language learning

To enable P3 gamut support these need to be set in `about:config`:

```
gfx.color_management.display_profile = /System/Library/ColorSync/Profiles/Display P3.icc
gfx.color_management.mode = 1
gfx.color_management.native_srgb = false
```

[consents]: https://addons.mozilla.org/en-US/firefox/addon/consent-o-matic/
[flagfox]: https://addons.mozilla.org/en-US/firefox/addon/flagfox/
[steamdb]: https://addons.mozilla.org/en-US/firefox/addon/steam-database/
[tamper]: https://addons.mozilla.org/en-US/firefox/addon/tampermonkey/
[ublock]: https://addons.mozilla.org/en-US/firefox/addon/ublock-origin/
[unhook]: https://addons.mozilla.org/en-US/firefox/addon/youtube-recommended-videos/
[yomitan]: https://addons.mozilla.org/en-US/firefox/addon/yomitan/
