---
title: A tale of Rust, Svelte, and islands of interactivity
date: 2025-07-30T12:00:13.020Z
tags: [svelte, rust, deno]
desc: >
  My attempt at calling Deno from Rust to prerender Svelte components via
  ESbuild, and bundling a client hydration script.
---

If you're building a modern website, you've probably felt the pull between two
different goals. On one hand, you need raw performance and fast load times,
which points toward server-side rendering. On the other, you want the dynamic,
app-like feel that client-side JavaScript frameworks provide.

Traditionally, you had to lean one way or the other. But what if you could
combine the strengths of both?

This post explores an idea which I had in my head for a few months: integrating
Svelte with Rust. I'll tackle the main challenge head-on: how to take a Svelte
component, render it to static HTML on the server using Rust, and then
seamlessly "hydrate" it on the client to bring it to life. This "islands of
interactivity" approach gives you the speed benefits of a static site with the
rich user experience of a single-page application.


### Getting Svelte to speak HTML

Before I could think about interactivity, I needed a way to simply turn a Svelte
component into static HTML during my Rust project's build process. This is the
core of Server-Side Rendering (SSR): create the HTML on the server so the
browser gets a fully-formed page right away. I started with a basic counter
button.

To simplify the basic proof of concept I started by handling the Svelte stuff in
the `build.rs` script. For the actual JavaScript bundling, I reached for
`esbuild` because of its incredible speed. The plan was to have the `build.rs`
script orchestrate esbuild to handle the Svelte compilation in two distinct
passes:

#### The server-side build:

The first task was to compile `Button.svelte` into code that could run on the
server (in a Node.js environment) and spit out raw HTML. A neat trick here was
encoding the bundle as URI component and immediately importing it to avoid
writing temporary files. The Rust build script tells Node to execute `esbuild`,
which compiles the Svelte component. The resulting JavaScript code is then
imported directly from the data, its render function is called, and the final
HTML is saved to a file like `Button.html`.

```typescript
// tools/svelte_bundle.mjs (SSR part - simplified)
// ...
const ssrBundle = await build({
    entryPoints: [file],
    bundle: true,
    format: "esm",
    platform: "node",
    write: false, // Don't write to disk yet
    plugins: [svelte({ compilerOptions: { generate: "ssr" } })],
});

// Import from data URI to get the SSR component
const { default: Comp } = await import(
    "data:text/javascript," + encodeURIComponent(ssrBundle.outputFiles[0].text)
);

// The Svelte SSR component's render function is a bit unusual
// It takes an object and populates the `out` key with the HTML
const data = { out: "" };
Comp(data);
const html = data.out;
// ... then this HTML is saved to Button.html
```

#### The client-side build

At the same time, a second esbuild process compiled the exact same
`Button.svelte` component, but this time for the browser. This created a
standard, self-contained JavaScript file (`Button.iife.js`) which holds all the
code needed for client-side interactivity.

Back in the Rust application, I used the `maud` templating library to assemble
the final page. The `include_str!` macro was perfect for this, letting me embed
the contents of the generated `Button.html` and its corresponding script
directly into the final Rust binary at compile time.

```rust
// src/templates.rs (initial approach)
use maud::{html, Markup, PreEscaped, DOCTYPE};

pub fn page() -> Markup {
    const BTN_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/Button.html"));
    const BTN_JS: &str = include_str!(concat!(env!("OUT_DIR"), "/Button.iife.js"));

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "Maud × Svelte Demo" }
            }
            body {
                // Directly inject the pre-rendered HTML
                (PreEscaped(BTN_HTML))

                // The script for hydration is included, but we're not
                // calling it just yet. One step at a time.
                // script type="module" { (PreEscaped(BTN_JS)) }
            }
        }
    }
}
```

Running the project confirmed the first success. The page loaded, and there was
my button, showing its initial state of "0". The server-side rendering worked.
But, of course, it was just an unresponsive and static button. We had our
island, but it was still lifeless, waiting for that spark of hydration.


### The hydration riddle

With the static HTML already rendered, the next crucial step was to infuse it
with interactivity. I uncommented the commented-out `<script>` tag, and reloaded
the page, however an error appeared in the browser's console:

```
Uncaught ReferenceError: Button is not defined
    <anonymous> http://127.0.0.1:3000/:2654
```

This error, a classic JavaScript scoping problem, revealed a fundamental
disconnect. Esbuild's IIFE bundle, designed for encapsulation, had effectively
hidden the `Button` component within its own private scope. The small, inline
JavaScript snippet, intended to kickstart hydration by instantiating `Button`,
simply couldn't find it on the global `window` object.

The immediate, seemingly logical fix was to use esbuild's `globalName` option.
By setting `globalName: 'SvelteButton'` in the client bundle configuration, the
`Button` component's default export should, in theory, be exposed globally as
`window.SvelteButton`.

```typescript
// tools/svelte_bundle.mjs (client part - attempting globalName)
// ...
await build({
  // ...
  format: 'iife',
  globalName: 'SvelteButton', // Expose the component globally
  // ...
  footer: {
    js: `
      const target = document.getElementById('button-root');
      if (target) {
        new SvelteButton({ target, hydrate: true }); // Attempt to use the global name
      }
    `
  }
});
```

The `ReferenceError` vanished as expected. A small victory. But this was merely
a prelude to a more insidious and cryptic error that quickly took its place:

```
Uncaught TypeError: can't access property "call", first_child_getter is undefined
    get_first_child http://127.0.0.1:3000/:960
    from_html http://127.0.0.1:3000/:2009
    Button http://127.0.0.1:3000/:2641
    <anonymous> http://127.0.0.1:3000/:2654
```

This error was an indicator of a deeper, architectural misunderstanding of
Svelte 5's hydration mechanism. It was attempting to "mount" a component (i.e.,
create new DOM nodes) onto a pre-existing DOM tree that was generated by the
server. The component's internal code, compiled without the proper hydration
flag, was expecting to build the DOM from scratch. When it encountered existing
nodes, its internal helper functions (like `first_child_getter`), crucial for
traversing and attaching to the existing DOM, had not been initialized. This led
to an immediate runtime crash.

The critical insight was this: for Svelte to correctly "hydrate" an existing DOM
tree, it needs to be explicitly told at compile time to generate a special kind
of bundle. One that understands how to attach itself to and update existing DOM
rather than simply creating a new one.

Initially, I tried simply passing `hydrate: true` to the component constructor.
However, Svelte 5's API changed: the component's default export is now a
function, not a class to be `new`-ed up. Correcting this to
`SvelteButton.default(target, { hydrate: true })` still didn't fix the core
`first_child_getter is undefined` issue. This strongly indicated that the
*compiled output itself* was not configured for hydration.

Next I tried looking into esbuild's `svelte` plugin options. I came across the
`compilerOptions.mode: 'hydrate'` flag, which instructs the Svelte compiler to
emit code specifically designed for attaching to and re-using existing DOM.

```javascript
// tools/svelte_bundle.mjs (client part - adding hydration mode)
// ...
await build({
  // ...
  plugins: [
    svelte({
      compilerOptions: {
        mode: 'hydrate'
      }
    })
  ],
  // ...
});
```

It instructed Svelte to generate the necessary internal hydration markers and
logic. Yet, even with `mode: 'hydrate'`, the *invocation* still needed to be
correct. Svelte 5 introduced a top-level `hydrate` function from its main
export.


### The Astro introspection

When faced with a fundamental challenge, the wisest approach is often to study
how other frameworks have already solved it. My investigation turned to
**Astro**, a framework famous for its "Islands Architecture," a pattern
precisely aligned with our goal: shipping minimal JavaScript while progressively
enhancing parts of the page. How did Astro achieve its seamless pre-rendering
and hydration of Svelte components?

Analyzing Astro's build output and source code proved to be a good learning
experience, with some key insights:

1. Astro `<astro-island>` wrapper

Astro doesn't merely dump raw HTML. Each interactive component is intelligently
wrapped in a custom `<astro-island>` HTML element. This element serves as a
manifest, containing vital metadata as HTML attributes: `component-url` (path to
the component's browser JavaScript), `renderer-url` (path to a shared Svelte
renderer/bootloader), `props` (serialized data for the component), and a
`client` directive (`client:load`, `client:idle`, `client:visible`,
`client:media`, or `client:only`) that dictates *when* the component should
hydrate.

```html
<!-- Example Astro output for a Svelte component -->
<astro-island
  uid="byYzy"
  component-url="/_astro/Test.dfb48ca8.js"
  component-export="default"
  renderer-url="/_astro/client.c4e17359.js"
  client="load"
  props='{"count":0}'
  ssr>
  <button data-h="byYzy">0</button>
</astro-island>
```

The inner `<button>` is the SSR'd content, immediately visible to users and
search engine bots without JavaScript. The `data-h` attribute contains Svelte's
internal hydration markers, which are emitted unconditionally in Svelte 5.

2. True two-bundle system

Similar to our initial attempt, Astro generates separate builds for the server
(for SSR) and the client (for hydration). However, the client-side bundle isn't
just the raw Svelte component.

3. Dedicated hydration entrypoint (bootloader)

This was the pivotal missing piece. Astro doesn't directly ship the raw Svelte
component's JavaScript to the browser for hydration. Instead, it ships a tiny,
framework-specific **bootloader** script (`renderer-url`). This bootloader's
sole responsibility is to orchestrate the hydration. When the `client:`
directive's trigger fires, the bootloader performs these critical steps:

  * Dynamically `import()` the shared Svelte renderer (if not already loaded).
  * Dynamically `import()` the specific Svelte component's browser bundle (`component-url`).
  * Calls the renderer's internal `hydrate()` helper, which is fundamentally:

```typescript
import { hydrate } from 'svelte'; // The crucial part!
// ...
hydrate(Component, {
  target: island.firstElementChild, // The pre-rendered DOM
  props: JSON.parse(island.getAttribute('props'))
});
```

The fundamental mistake wasn't just in esbuild's `globalName` or `mode` flags;
it was in the entire *orchestration*. I was attempting to make the Svelte
component hydrate itself from an inline script. The correct pattern, as
demonstrated by Astro, was to have an external orchestrator (a dedicated
entrypoint script) manage the hydration process by explicitly calling
`hydrate()` from Svelte's runtime. This external call correctly initializes
Svelte's internal state (like `first_child_getter`), allowing it to attach to
the existing DOM without errors.


### Deno detour

Armed with this critical new understanding, the project embarked on a slight
detour: migrating the build script from Node.js to **Deno**. Deno, a modern and
secure runtime for JavaScript and TypeScript, offered compelling advantages in
terms of built-in tooling, security, and a cleaner programming model. I decided
to go with it mainly because Deno allows specifying dependencies from `npm`
directly in import statements.

My first attempt to integrate the Astro-inspired bootloader involved a "virtual"
entrypoint. The idea was to generate the simple hydration script as a string in
memory and feed it directly to esbuild by faking the resolution process.

```typescript
import { build } from "npm:esbuild";
import svelte from "npm:esbuild-svelte";

const componentPath = Deno.args[0]; // The .svelte file
const VIRTUAL_ENTRY = "virtual-entry.js";

// Virtual entry module content
const entryContents = `
  import { hydrate } from "svelte";
  import Component from ${JSON.stringify(componentPath)};

  const target = document.getElementById("button-root");
  if (target) hydrate(Component, { target });
`;

const result = await build({
  entryPoints: [VIRTUAL_ENTRY],
  bundle: true,
  format: "esm",
  platform: "browser",
  write: false,
  plugins: [
    {
      name: "virtual-entry",
      setup(build) {
        build.onResolve({ filter: /^virtual-entry\.js$/ }, () => ({
          path: VIRTUAL_ENTRY,
          namespace: "virtual",
        }));

        build.onLoad({ filter: /.*/, namespace: "virtual" }, () => ({
          contents: entryContents,
          loader: "js",
        }));
      },
    },
    svelte({
      compilerOptions: {
        css: "external",
      },
    }),
  ],
});
```

This idea quickly led to a fresh set of cryptic errors:

```
✘ [ERROR] Could not resolve "svelte"
    virtual:___virtual.js:2:36:
      2 │             import { hydrate } from "svelte";
        ╵                                     ~~~~~~~~

  The plugin "virtual" didn't set a resolve directory for the file "virtual:___virtual.js", so esbuild did not search for "svelte" on the file system.

✘ [ERROR] Could not resolve "js/search/src/App.svelte"
    virtual:___virtual.js:3:34:
      3 │             import Component from "js/search/src/App.svelte";
        ╵                                   ~~~~~~~~~~~~~~~~~~~~~~~~~~

  The plugin "virtual" didn't set a resolve directory for the file "virtual:___virtual.js", so esbuild did not search for "js/search/src/App.svelte" on the file system.
```

I didn't know enough about the esbuild build process to be able to resolve this
issue, and it seemed too brittle anyway, so I decided to try a different
approach.


### More robust architecture

Armed with the hard-won lessons from Astro and the battle scars from esbuild
module resolution quirks, the final, elegant, and robust solution began to take
shape. It seamlessly integrates Rust's build system with Deno's runtime, all
while adhering to the core architectural needs of Svelte's hydration model.

Here's the detailed anatomy of the final implementation:

#### Simplicity on the surface

I've integrated this solution into my static site generator `hauchiwa`. The
complexity of the underlying build process is encapsulated behind a clean,
high-level Rust API: `glob_svelte`. This function provides a simple interface
for users to register Svelte components within their Rust-powered application.

```rust
// src/loader/mod.rs (Simplified for clarity)

// Represents a pre-rendered Svelte component with client-side hydration.
pub struct Svelte<P = ()>
where
    P: serde::Serialize,
{
    /// Function that renders the component to an HTML string given props.
    pub html: Prerender<P>, // Prerender is a Box<dyn Fn(&P) -> Result<String> + Send + Sync>

    /// Path to a JavaScript file that bootstraps client-side hydration.
    pub init: Utf8PathBuf, // Points to a file in the build cache
}

/// Constructs a loader that processes Svelte components into pre-renderable assets.
pub fn glob_svelte<P>(path_base: &'static str, path_glob: &'static str) -> Loader
where
    P: serde::Serialize + 'static,
{
    // This `Loader` is an implementation detail from my SSG library you can ignore this...
    Loader::with(move |_| {
        LoaderGenericMultifile::new(
            path_base,
            path_glob,
            |path| {
                // Phase 1: Compile SSR bundle & generate a unique ID for the component
                let server_bundle_text = compile_svelte_server(path)?;
                let anchor_hash_id = Hash32::hash(server_bundle_text.as_bytes());

                // Phase 2: Compile client hydration bundle
                let client_bundle_text = compile_svelte_init(path, anchor_hash_id)?;

                // Calculate a content hash for the client bundle for caching
                let client_bundle_hash = Hash32::hash(client_bundle_text.as_bytes());

                // Create the HTML rendering closure
                let html_renderer = Box::new({
                    let anchor_id_hex = anchor_hash_id.to_hex();
                    let server_bundle_text_cloned = server_bundle_text.clone();

                    move |props: &P| {
                        let json_props = serde_json::to_string(props)?;
                        // Use the previously compiled SSR bundle and dynamic props to render HTML
                        let rendered_html = run_ssr(&server_bundle_text_cloned, &json_props)?;
                        // Wrap with a unique class for client-side targeting and embed props
                        Ok(format!("<div class='_{anchor_id_hex}' data-props='{json_props}'>{rendered_html}</div>"))
                    }
                });

                // Return the hash of the client bundle and a tuple of the html_renderer and client bundle text
                Ok((client_bundle_hash, (html_renderer, client_bundle_text)))
            },
            |rt, (html_renderer, client_bundle_text)| {
                // Store the client bundle text to disk in the build cache
                let client_init_path = rt.store(client_bundle_text.as_bytes(), "js")?;
                // Return the Svelte struct
                Ok(Svelte { html: html_renderer, init: client_init_path })
            },
        )
    })
}
```

#### Rust orchestrates Deno

The core of the Svelte integration lies in three distinct Deno calls,
meticulously orchestrated by Rust's `Command` API. This pattern allows us to
delegate the complex JavaScript bundling tasks to Deno (and esbuild) while
maintaining full control and error handling within Rust.

1. `compile_svelte_server(file: &Utf8Path) -> anyhow::Result<String>`

This function is responsible for compiling the Svelte component into its
Server-Side Renderable (SSR) JavaScript bundle. Instead of writing the bundle to
a file, esbuild is configured to output directly to `stdout`. The Rust `Command`
captures this output.

```rust
// src/loader/mod.rs
fn compile_svelte_server(file: &Utf8Path) -> anyhow::Result<String> {
    const JS: &str = r#"
        import { build } from "npm:esbuild@0.25.6";
        import svelte from "npm:esbuild-svelte@0.9.3";
        import * as path from "node:path"; // Use Node.js path module for Deno

        const componentFilePath = Deno.args[0]; // Absolute path from Rust

        const ssr = await build({
            entryPoints: [componentFilePath], // Use the absolute path directly
            format: "esm",
            platform: "node", // Compile for Node.js environment
            minify: true,
            bundle: true,
            write: false, // Output to memory
            mainFields: ["svelte", "module", "main"],
            conditions: ["svelte"],
            plugins: [
                svelte({
                    compilerOptions: { generate: "server" }, // Generate SSR code
                    css: false, // Don't emit CSS as separate file from SSR
                    emitCss: false, // Crucial to prevent "fakecss:" errors for SSR
                }),
            ],
        });

        // The SSR bundle's text is URI-encoded for safe transport via stdout
        const encodedText = encodeURIComponent(ssr.outputFiles[0].text);
        await Deno.stdout.write(new TextEncoder().encode(encodedText));
        await Deno.stdout.close();
    "#;

    // ... Rust Command invocation for Deno, capturing stdout and stderr ...
    // Use `file.canonicalize()?` to ensure absolute path for Deno.args[0]
}
```

Notice the `encodeURIComponent` on the SSR bundle's text. This is a crucial
trick. The raw SSR JavaScript might contain characters that could interfere with
command-line arguments or `data:` URIs. By encoding it, we ensure safe
transmission from the first Deno process to the second, which will import it.
`platform: "node"` is used to ensure the Svelte compiler produces code
compatible with a Node.js-like environment, even though we're running it within
Deno's Node.js compatibility layer.

2. `run_ssr(server_bundle: &str, props_json: &str) -> anyhow::Result<String>`

This function takes the URI-encoded SSR bundle (obtained from
`compile_svelte_server`) and the component's `props` (as a JSON string). It
spawns another Deno process, which then imports the SSR bundle via a `data:` URI
and calls its `render` function with the provided props to produce the final
HTML.

```rust
// src/loader/mod.rs
fn run_ssr(server_bundle: &str, props: &str) -> anyhow::Result<String> {
    // The Deno script that imports the SSR bundle and renders it
    let js_runner = format!(
        r#"
        const jsonProps = Deno.args[0];
        const parsedProps = JSON.parse(jsonProps);

        // Import the SSR bundle directly from the data URI
        const {{ default: SSRComponent }} = await import("data:text/javascript,{}");

        let renderedOutput = null;

        // Svelte 5 SSR might return different structures (e.g., { out: [] } or { out: "" })
        // This attempts both common patterns for flexibility.
        try {{
            const data = {{ out: [] }}; // For components that render to an array of nodes
            SSRComponent(data, parsedProps);
            renderedOutput = data.out.join('');
        }} catch (e) {{
            // Fallback for components rendering directly to a string
            try {{
                const data = {{ out: "" }};
                SSRComponent(data, parsedProps);
                renderedOutput = data.out;
            }} catch (e2) {{
                throw new Error("Failed to produce prerendered component. Original: " + e.message + " Fallback: " + e2.message);
            }}
        }}

        if (renderedOutput === null) {{
            throw new Error("Failed to produce prerendered component, check Svelte 5 SSR usage.");
        }}

        await Deno.stdout.write(new TextEncoder().encode(renderedOutput));
        await Deno.stdout.close();
    "#,
        server_bundle // The URI-encoded bundle from compile_svelte_server
    );

    // ... Rust Command invocation for Deno, passing props as Deno.args[0] ...
    // Crucially, this Deno process doesn't need --allow-read because it's working with in-memory data.
}
```

The `run_ssr` function includes a `try...catch` block. This is a subtle but
important detail for Svelte 5. Depending on how a Svelte 5 component is
structured and compiled for SSR (especially with or without slots), its `render`
function (accessed via `Comp(data, props)`) might populate `data.out` as an
array of strings or a single string. The `try...catch` gracefully handles both
scenarios, making the SSR rendering more robust. This also demonstrates how you
can pass *dynamic* props from your Rust application to the Svelte component at
render time.

3. `compile_svelte_init(file: &Utf8Path, anchor_hash_id: Hash32) -> anyhow::Result<String>`

This function is the culmination of all the hydration struggles. It compiles the
client-side JavaScript bundle responsible for hydration. Instead of generating a
temporary file for the entrypoint, it constructs a "stub" entrypoint as a string
and feeds it to esbuild's `stdin` option.

```rust
// src/loader/mod.rs
fn compile_svelte_init(file: &Utf8Path, hash_class: Hash32) -> anyhow::Result<String> {
    let abs_file_path = file.canonicalize()?; // Resolve to absolute path on Rust side
    let hash_hex = hash_class.to_hex();

    // The in-memory "stub" entrypoint for client-side hydration
    let stub_entrypoint = format!(
        r#"
        import {{ hydrate }} from "svelte"; // Svelte 5's global hydrate function
        import App from {}; // Import the Svelte component using its absolute path

        const query = document.querySelectorAll('._{}'); // Target elements by unique class
        for (const target of query) {{
            const attrs = target.getAttribute('data-props');
            const props = JSON.parse(attrs) ?? {{}}; // Parse props from data-attribute
            hydrate(App, {{ target, props }}); // Hydrate each instance
        }}
    "#,
        // Use JSON.stringify for safe embedding of the path into JS string
        serde_json::to_string(&abs_file_path.to_string_lossy())?,
        hash_hex // The unique class ID generated from SSR bundle's hash
    );

    const JS_RUNNER: &str = r#"
        import * as path from "node:path";
        import { build } from "npm:esbuild@0.25.6";
        import svelte from "npm:esbuild-svelte@0.9.3";

        const stubContents = Deno.args[0]; // The entrypoint stub from Rust
        const componentFilePath = Deno.args[1]; // The original component path, for resolveDir

        const ssr = await build({
            stdin: {
                contents: stubContents,
                // Critical: resolve relative imports from the original component's directory
                resolveDir: path.dirname(path.resolve(componentFilePath)),
                sourcefile: "__virtual_client_entry.ts", // For better debug traces
                loader: "ts", // Use TS loader if you're using `!` non-null assertions etc.
            },
            platform: "browser",
            format: "esm",
            bundle: true,
            minify: true,
            write: false, // Output to memory
            mainFields: ["svelte", "browser", "module", "main"],
            conditions: ["svelte", "browser"],
            plugins: [
                svelte({
                    compilerOptions: {
                        css: "external",
                    },
                }),
            ],
        });

        await Deno.stdout.write(new TextEncoder().encode(ssr.outputFiles[0].text));
        await Deno.stdout.close();
    "#;

    // ... Rust Command invocation for Deno, passing stub_entrypoint and abs_file_path ...
}
```

* **Rust-side path canonicalization:** The component's file path is
  canonicalized to an absolute path (`file.canonicalize()?`) in Rust. This
  ensures absolute path stability regardless of the Rust process's CWD. The
  absolute path is then passed to the Deno subprocess.
* **In-memory stub:** The `stub_entrypoint` string is built in Rust, containing
  the `hydrate` import and the component import. The component's absolute path
  is embedded directly into this stub, removing any ambiguity for esbuild.
* **`stdin` for esbuild:** The Deno subprocess then uses esbuild's `stdin`
  feature to feed this `stub_entrypoint` directly to the bundler.
* **`resolveDir` is paramount:** Even with an absolute component path,
  `resolveDir` is set to the `dirname` of the original component's absolute path
  (`path.dirname(path.resolve(componentFilePath))`). This is still crucial for
  `esbuild` to correctly resolve any other relative imports *within* the Svelte
  component itself (e.g., if `App.svelte` imports `./store.js`).
* **Querying by class:** Instead of targeting a single `id`, the
  `stub_entrypoint` uses `document.querySelectorAll('._${hash}')`. This allows
  for *multiple instances* of the same Svelte component on a single page, each
  independently hydrated. The `_` prefix for the class is a convention to avoid
  creating invalid CSS class names and to signify its internal purpose.
* **`data-props` for per-instance data:** The `data-props` attribute on the
  SSR'd `div` is critical. It allows each hydrated instance to receive its
  unique set of props from the server, enabling dynamic content and behavior
  without additional network requests. The client-side JavaScript then
  `JSON.parse`s this attribute.


#### Bringing it all together

The `html` closure within the `Svelte<P>` struct is where the final pieces of
the puzzle align. It creates the final piece of HTML, which contains prerendered
component, as well as props and hash class.

```rust
// Inside the glob_svelte loader, within the mapping function
// ...
let html = Box::new({
    let anchor = anchor_hash_id.to_hex();
    let server_bundle_text_cloned = server_bundle_text.clone();

    move |props: &P| {
        let json = serde_json::to_string(props)?;
        let html_content = run_ssr(&server_bundle_text_cloned, &json)?;
        // Render the wrapper div with a unique class and data-props
        Ok(format!(
            "<div class='_{anchor}' data-props='{}'>{html_content}</div>",
            &json
        ))
    }
});

// ... later, in the template ...
// The `Svelte` struct `my_svelte_component` would be retrieved from loader
let Svelte { html: render_html_fn, init: init_js_path } = my_svelte_component;

// Call the function to get the pre-rendered HTML for this instance
let rendered_component_html = render_html_fn(&my_props_instance)?;

html! {
    (DOCTYPE)
    html {
        head {
            meta charset="utf-ob";
            title { "My Awesome Svelte App" }
        }
        body {
            (PreEscaped(rendered_component_html)) // Inject the SSR'd HTML
            script type="module" src=(init_js_path) {} // Link to the hydration script
        }
    }
}
```

When the browser receives this HTML:
* The static content (e.g., `<button>0</button>`) is immediately displayed.
* The `<script type="module" src="..."` loads the `init` JavaScript bundle.
* This script executes, finds all `div`s with the unique class (e.g.,
  `_a1b2c3d4`), parses their `data-props`, and then calls `hydrate(App, {
  target, props })` for each.
* Svelte takes over, attaching event listeners and making the component
  interactive, without re-rendering the entire DOM.


### Some future considerations

The journey to a fully hydrated Svelte component in a Rust application is
complete. However, the path of web development is ever-evolving. Here are some
advanced considerations and future directions for this architecture:

* **CSS Handling:** I haven't yet figured out how to process the CSS which is
  included directly in Svelte files, so this would have to be done to be able to
  use all the features of Svelte. For now I was just writing CSS separately from
  Svelte.
* **Source Maps:** For effective debugging of client-side Svelte code,
  generating and serving source maps is essential. Esbuild has options for this
  (`sourcemap: true`). I'd probably need to store these maps alongside `.js`
  files and configure the HTTP server to serve them correctly.
* **Progressive Hydration Strategies:** Astro's `client:idle`, `client:visible`,
  `client:media` directives offer more granular control over when hydration
  occurs. Maybe it would be possible to extend the `compile_svelte_init` to
  generate different hydration stubs based on a `HydrationStrategy` enum passed
  from Rust.


### Conclusion

My quest to integrate Svelte with a Rust backend, aiming for truly interactive
"islands" on a fast, server-rendered site, led me through some significant
hurdles. The core challenge wasn't just getting Svelte to spit out HTML, but
making its client-side code gracefully "hydrate" that pre-existing HTML without
errors.

My breakthrough came from dissecting how frameworks like Astro achieved this
seamless pre-rendering and hydration. It wasn't about a single magic flag, but a
sophisticated orchestration: a dedicated bootloader script and Svelte's hydrate
function. This realization reshaped my entire build process.

I ended up building a robust system where Rust commands Deno to handle the
complex Svelte compilation, both for server-side rendering and client-side
hydration. By carefully passing component paths and props between these
environments, I created a pipeline that generates static HTML first, then
precisely attaches the necessary JavaScript for interactivity. This "islands of
interactivity" approach now gives me the best of both worlds: the raw
performance of server-side rendering and the rich, dynamic feel of a Svelte
application.
