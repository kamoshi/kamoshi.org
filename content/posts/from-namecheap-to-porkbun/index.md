---
title: From NameCheap to PorkBun
date: 2025-04-23T22:28:37.720Z
desc: >
  Transferring a domain name sounds difficult, but is it? Not at all...
---

There are a couple of reasons why I ended up transferring my domain. The first
one is price. On NameCheap, renewing a .org domain costs $15.16, while on
PorkBun it's only $10.72. The second reason is that PorkBun generally has better
reviews. I kept seeing people recommend it on sites like Hacker News and Reddit.
But the third reason (and probably the most important) is that I just wanted to
see what it's like to transfer a domain between registrars. I've never done it
before, so it felt like a good opportunity to learn something new. Plus, it
might come in handy if I ever need to do it again in the future.

## How to do it

For the sake of retaining the valuable knowledge, I'll describe the things I had
to do, to transfer the domain between registrars.

The first thing you can do is add an "external domain" to PorkBun. I'm not sure
if this is specific to PorkBun or not, but it makes it easy to prepare the
infrastructure in advance.

To do this, you have to add a `TXT` record with a `bun-verify` key. This lets
PorkBun verify that you actually own the domain.

Once the domain is added, you can fill out the DNS entries in PorkBun and enable
DNS in the domain settings. This won't actually change anything yet, it just
lets you mirror your existing DNS setup ahead of time.

Make sure the DNS settings in PorkBun match the old DNS exactly, for example the
ones on NameCheap.

Once that's done, go to NameCheap and point your domain to the PorkBun
nameservers. At the time of writing, these are:

- `curitiba.ns.porkbun.com`
- `fortaleza.ns.porkbun.com`
- `maceio.ns.porkbun.com`
- `salvador.ns.porkbun.com`

After updating the nameservers, you should check if everything is working
correctly using a [DNS checker](https://www.whatsmydns.net/). Pay special
attention to the `NS`, `A`, and `MX` records to make sure websites and email
will still work.

At this point, everything is reversible. If something doesn't work, you can go
back into NameCheap and switch to the old DNS settings.

If everything looks good, you can move on to transferring the domain name.

From here, it's mostly a matter of following PorkBun's official guides (check
the bibliography at the end). You'll need to:

- Unlock the domain in NameCheap
- Generate an auth code
- Enter the auth code on PorkBun
- Confirm the transfer via an email from NameCheap

After that, you'll get another email from NameCheap letting you speed up the
transfer. Instead of waiting 5 days, you can approve it and have the transfer
finish in about 15 minutes.


Once that's done, your domain should be fully transferred with zero downtime.
The key is making sure the DNS setup is in place beforehand. Alternatively, you
could keep your DNS completely independent from either registrar, which works
too.
