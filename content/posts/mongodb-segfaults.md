---
title: MongoDB kept segfaulting and Gemini saved me
date: 2026-03-10T19:40:18.227Z
desc: >
  A local MongoDB instance started randomly segfaulting with no clear cause, no
  fatal logs, no data corruption, Docker made no difference. An hour of
  debugging with Gemini eventually uncovered something unexpected.
---

My local MongoDB instance seemingly randomly started crashing today, so I had to
spend an hour debugging it with Gemini, writing this up here for posterity.

## What I saw

`systemctl status mongodb.service` showed this:

```
× mongodb.service - MongoDB Database Server
     Active: failed (Result: core-dump) since Tue 2026-03-10 09:32:44 CET
    Process: 1064 ExecStart=/usr/bin/mongod --config /etc/mongodb.conf (code=dumped, signal=SEGV)
   Main PID: 1064 (code=dumped, signal=SEGV)
```

The C++ stack trace pointed to `JournalFlusher::run()` which is WiredTiger's
thread for flushing write-ahead logs to disk. While crashing, MongoDB tried to
log what went wrong through `boost::log`, hit another memory error in the
process, and that's what generated the dump.

I restarted the service and it came back up. Connected with `mongosh`, ran some
commands, everything seemed fine. Then about a minute later, mid-session:

```
MongoServerSelectionError: connect ECONNREFUSED 127.0.0.1:27017
```

Dead again and it was the same SEGV, same thread.

## First theory: ABI mismatch

The MongoDB log had no `FATAL` message before dying, as it just stopped. But
buried in the startup output was this:

```json
"buildInfo": { "environment": { "distmod": "ubuntu2404" } }
```

I'm on Arch Linux, so I installed MongoDB via the `mongodb80-bin` AUR package,
which is a pre-compiled Ubuntu 24.04 binary. Arch is rolling release, so core
C++ libraries like `boost` get updated regularly, and the crash was consistently
happening inside `boost::log`, which seemed to point towards an ABI mismatch.
The binary was built against older Ubuntu versions of those libraries, so a
recent system update could have broken compatibility.

Gemini suggested switching to Docker to sidestep the library issue entirely,
since the official `mongo:8.0` image bundles the Ubuntu environment the binary
was built for.

## Docker didn't help

I stopped and disabled the systemd service, then ran:

```bash
docker run --name mongodb -d -p 27017:27017 -v /var/lib/mongodb:/data/db mongo:8.0
```

Container came up, startup looked clean at first, connected with `mongosh`...
then `ECONNREFUSED` again. `docker ps -a` showed `Exited (139)`.

The container was crashing with the exact same segfault.

I changed the Docker command a bit to fix a few issues: my config had `bindIp:
127.0.0.1` which inside a container means the container's own localhost (not the
host machine), and the default data path `/data/db` didn't match what my config
file specified. Also needed `--user 966:966` to match the `mongodb` user's UID
on Arch so the container could write to the host files. Final command:

```bash
docker run --name mongodb -d \
  -p 27017:27017 \
  --user 966:966 \
  -v /etc/mongodb.conf:/etc/mongodb.conf:ro \
  -v /var/lib/mongodb:/var/lib/mongodb \
  -v /var/log/mongodb:/var/log/mongodb \
  mongo:8.0 \
  mongod --config /etc/mongodb.conf --bind_ip_all
```

Same result, it crashed within seconds of connecting.

## Maybe the data files are corrupted?

The database had previously been initialized as a replica set and then abandoned
without a clean shutdown. Gemini suggested this could leave WiredTiger in a
state that causes a hard crash in 8.0 when it tries to flush journal entries. I
could easily rule it out by giving MongoDB a fresh start.

I moved the old data directory out of the way:

```bash
sudo mv /var/lib/mongodb /var/lib/mongodb_old
sudo mkdir /var/lib/mongodb
sudo chown mongodb:mongodb /var/lib/mongodb
```

I relaunched the container with the same command, but then it crashed again with
the exact same exit code 139. An empty database inside a clean Docker container
was segfaulting, so it wasn't the data, wasn't the Arch libraries, wasn't the
config...

## What was actually killing it

This is something I wouldn't have ever thought of, unless Gemini told me about it.

Going back to the `dmesg` output I'd checked earlier, I had a modern AMD CPU and
looking at the very first crash stack trace again, one of the frames was
`_ZN5mongo9transport15SessionWorkflow`. This is MongoDB's session handling code,
which uses C++ coroutines that work by rapidly swapping memory stacks.

Gemini flagged something I had no idea about: recent AMD CPUs with newer Linux
kernels enable **Control-flow Enforcement Technology (CET)**, specifically
**Shadow Stacks (SHSTK)**. It's a hardware security feature that monitors
call/return patterns to detect ROP attacks via gadget chains. When MongoDB's
session handler swaps its coroutine stack, the CPU sees a stack pointer jumping
to an unexpected location and treats it as an attack, killing the process with a
SIGSEGV before MongoDB can even write a single log entry.

This would explain everything:
- Crashes right after a client connects (session creation triggers the coroutine stack swap)
- No `FATAL` log message (hardware kill, no time to log anything)
- Docker making no difference (containers share the host kernel and CPU)
- A fresh empty database also crashing (nothing to do with data state)

The fix is to tell `glibc` to disable Shadow Stacks for the process:

```bash
GLIBC_TUNABLES=glibc.cpu.hwcaps=-SHSTK
```

For Docker:

```bash
docker run --name mongodb -d \
  -p 27017:27017 \
  -e GLIBC_TUNABLES="glibc.cpu.hwcaps=-SHSTK" \
  --user 966:966 \
  -v /etc/mongodb.conf:/etc/mongodb.conf:ro \
  -v /var/lib/mongodb:/var/lib/mongodb \
  -v /var/log/mongodb:/var/log/mongodb \
  mongo:8.0 \
  mongod --config /etc/mongodb.conf --bind_ip_all
```

For systemd, via an override so it survives package updates:

```bash
sudo systemctl edit mongod
```

```ini
[Service]
Environment="GLIBC_TUNABLES=glibc.cpu.hwcaps=-SHSTK:glibc.pthread.rseq=0"
```

The second tunable `glibc.pthread.rseq=0` was already being set by something
else in my systemd config. It disables Restartable Sequences, apparently needed
in some virtualization setups. Since `GLIBC_TUNABLES` is a single
colon-separated string, I had to combine them or one would shadow the other.

After restarting MongoDB stayed up and when I ran commands it didn't crash.

I'm not 100% certain the SHSTK explanation is correct. I didn't go deep enough
into the crash to be sure that's what's happening, but adding that tunable
stopped the segfaults, so that's where I'm leaving it.

Some more links which might be relevant:
- <https://jira.mongodb.org/browse/SERVER-120238>
- <https://github.com/docker-library/mongo/discussions/748>
