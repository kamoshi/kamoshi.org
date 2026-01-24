---
title: Todo
---

## Markdown parser

1. directives

The old implementation had a robust custom directive system. The new
implementation currently relies on standard Markdown or shortcodes, but lacks
the custom handlers.

- [ ] Port sidenote container: The `<aside class="marginnote">` logic is missing.
- [ ] Port youtube block: The iframe injection logic is missing.
- [ ] Port icon inline directive: The `<img class="inline-icon">` logic is missing.
- [ ] Port cite inline directive: The old code hijacked InlineMath for
  citations; this logic is absent.

2. bibliography and citations (hayagriva)

The entire bibliography subsystem is currently missing from the comrak
implementation.

- [ ] Integrate hayagriva library: Pass the Library struct to the parser.
- [ ] Citation scanning: Implement a pass (likely AST traversal) to identify
  citation keys (e.g., [@key] or the old cite directive).
- [ ] Bibliography Rendering: Generate the HTML bibliography list and append it
  to the document.

3. outline

The old parser returned an Article struct containing the TOC.

- [ ] Extract Table of Contents

4. special code block handling

- [ ] Restore typst support: The old code checked for
  CodeBlockKind::Fenced("typst svg") and rendered it using typst::render_typst.
  The new code currently sends everything to the syntax highlighter.


## Hauchiwa

### Robustness & developer experience

*Goal: Create a tool that is impossible to hold wrong and provides helpful feedback.*

- [ ] **1. Eliminate Panic-Based Error Handling**
    * **Context:** Currently, `run_once_parallel` panics if a graph cycle is
      detected (`toposort(...).expect(...)`) or if channel communication fails
      (`unwrap()`).
    * **Action:** Refactor `executor.rs` to catch `toposort` errors and return a
      `BuildError::CycleDetected` with a formatted dependency chain. Ensure all
      thread-pool communication uses `?` propagation to cleanly shut down
      workers on failure.
    * **Benefit:** Prevents the library from crashing the host process, enabling
      integration into long-running servers or IDE plugins.

- [ ] **2. External Toolchain Pre-flight Checks**
    * **Context:** Loaders for JS and Svelte invoke `Command::new("esbuild")`
      and `Command::new("deno")` blindly. If missing, the build fails with a
      confusing `IO Error: No such file` inside a worker thread.
    * **Action:** Implement a `HealthCheck` trait for tasks. Before the build
      graph starts, query the versions of all required external binaries (e.g.,
      `esbuild --version`). Fail fast with a helpful message ("Please install
      esbuild") if they are missing.
    * **Benefit:** "It just works" experience for new users cloning a repo.

- [ ] **3. Diagnostic Error Reporting (Miette Integration)**
    * **Context:** Errors are currently simple strings or `anyhow` wrappers.
    * **Action:** Integrate `miette` or `codespan-reporting`. When a frontmatter
      parse fails or a SCSS compilation errors, display the specific file path,
      line number, and a snippet of the offending code code.
    * **Benefit:** Drastically reduces debugging time by pointing users exactly
      to the typo.

- [ ] **4. Path Traversal Safety Rails**
    * **Context:** The `Output` struct allows arbitrary paths, and
      `save_pages_to_dist` blindly joins them. `Output::file("/etc/passwd",
      ...)` would write outside the build directory.
    * **Action:** Sanitize `page.url` in `save_pages_to_dist`. If a path
      resolves to a parent directory of `dist`, reject it with
      `BuildError::SecurityPolicyViolation`.
    * **Benefit:** Protects developers from accidental configuration errors that
      could overwrite source files or system config.

- [ ] **5. Structured Telemetry & Tracing**
    * **Context:** Observability is limited to `println!` statements and a
      custom SVG generator.
    * **Action:** Replace `println!` with the `tracing` crate. Implement a
      `ChromeLayer` to export build performance data to `trace.json` (compatible
      with `chrome://tracing` and standard profilers).
    * **Benefit:** Allows users to visualize exactly which tasks (Svelte
      compilation, Image resizing) are slowing down their build.

### Performance & efficiency

*Goal: Maximize resource usage and minimize latency.*

- [ ] **6. Lazy Image Decoding & Metadata Caching**
    * **Context:** `process_image` decodes the *entire* source image into memory
      (`img.decode()?`) *before* checking if the optimized version already
      exists in the cache. This makes incremental builds (cold start) extremely
      slow as every image is re-read and re-decoded.
    * **Action:** Move the decoding logic *inside* the cache-miss block. Cache
      image dimensions (width/height) in a lightweight sidecar file (e.g.,
      `.meta.json`) so the `Image` struct can be returned without reading the
      pixel data.
    * **Benefit:** Makes "second run" builds nearly instant, even with thousands
      of images.

- [ ] **7. Atomic & Differential Output Sync**
    * **Context:** `clear_dist()` wipes the entire output directory at the start
      of every build. This thrashes the SSD and causes 404s for live-reload
      servers.
    * **Action:** Deprecate `clear_dist`. Implement a "sync" strategy: Overwrite
      files only if content differs. Delete only orphaned files at the end of
      the build.
    * **Benefit:** Zero-downtime hot reloading and reduced disk wear.

- [ ] **8. Optimized I/O (Hard Links & Reflinks)**
    * **Context:** Artifacts are written to a hidden `.cache` and then
      physically copied to `dist` using `fs::copy`. This doubles I/O bandwidth
      usage.
    * **Action:** Modify `Store::save` to attempt `fs::hard_link` first. If that
      fails (cross-device), fall back to `fs::copy`.
    * **Benefit:** Halves the time spent writing assets to disk on supported
      filesystems.

- [ ] **9. Adaptive Hashing Strategy**
    * **Context:** `Hash32` uses `blake3` with `update_mmap_rayon` for all
      files. The overhead of memory-mapping and thread-pool synchronization is
      slower than simple `read()` for small files (< 1MB).
    * **Action:** Implement a heuristic: Use simple single-threaded hashing for
      small files; switch to parallel mmap hashing only for large assets
      (videos, hi-res images).
    * **Benefit:** Reduces CPU overhead for projects with many small text/JS
      files.

- [ ] **10. Persistent Subprocess Pooling**
    * **Context:** The Svelte loader spawns a new `deno` process for *every
      single component*. Process startup latency dominates the build time for
      sites with many components.
    * **Action:** Implement a long-lived `deno` worker pool. Spawn a few
      instances at the start and reuse them by feeding compilation requests via
      stdin/stdout or a local socket.
    * **Benefit:** Orders of magnitude faster compilation for component-heavy
      sites.
