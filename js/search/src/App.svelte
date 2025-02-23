<script lang="ts">
    import type { ChangeEventHandler, UIEventHandler } from "svelte/elements";

    let client = $state<Pagefind>();
    let query = $state<string>("");
    let limit = $state<number>(10);
    let result = $derived(client?.search(query));

    const require = (path: string) => import(/* @vite-ignore */ path);

    function sync(): void {
        query = new URLSearchParams(window.location.search).get("q") || "";
    }

    function onInput(): ChangeEventHandler<HTMLInputElement> {
        let debounce: number | undefined;

        return (event) => {
            clearTimeout(debounce);
            const value = event.currentTarget.value;
            const url = new URL(window.location.href);
            value
                ? url.searchParams.set("q", value)
                : url.searchParams.delete("q");
            debounce = setTimeout(
                () => window.history.pushState({}, "", url),
                1000,
            );
        };
    }

    function onScroll(): UIEventHandler<Window> {
        let throttle = Date.now();

        return (event) => {
            const now = Date.now();
            if (throttle + 200 > now) return;

            const { scrollHeight } = document.documentElement;
            const { innerHeight, scrollY } = event.currentTarget;

            const distance = scrollHeight - (innerHeight + scrollY);
            if (distance < 100) {
                limit += 5;
                throttle = now;
            }
        };
    }

    $effect(() => {
        sync();
        require("/pagefind/pagefind.js").then((pf) => (client = pf));
    });
</script>

<svelte:window on:popstate={sync} on:scroll={onScroll()} />

{#snippet tile(data: PagefindDocument)}
    <a class="c-search__result" href={data.url}>
        <header class="c-search__header">
            <h2 class="c-search__title">
                {data.meta.title}
            </h2>
        </header>
        <div class="c-search__excerpt">
            {@html data.excerpt}
        </div>
    </a>
{/snippet}

<article class="c-search">
    <h1>Search</h1>
    <input
        class="c-search__input"
        placeholder="Start typing here!"
        bind:value={query}
        oninput={() => (onInput(), (limit = 10))}
    />

    {#if query && result}
        {#await result}
            Loading...
        {:then { results }}
            <section class="c-search__results">
                <div>Showing results for "{query}" ({results.length})</div>
                {#each results.slice(0, limit) as page (page.id)}
                    {#await page.data()}
                        Loading...
                    {:then page}
                        {@render tile(page)}
                    {/await}
                {/each}
            </section>
        {/await}
    {:else}
        <div>No results to show yet...</div>
    {/if}
</article>
