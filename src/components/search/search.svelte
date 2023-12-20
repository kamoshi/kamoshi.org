<script lang="ts">
import type { ChangeEventHandler } from 'svelte/elements';

let client = $state<Pagefind>();
let query  = $state<string>('');
let result = $derived(client?.search(query));


function handler(): ChangeEventHandler<HTMLInputElement> {
  let debounce: number | undefined;

  return event => {
    clearTimeout(debounce);
    const value = event.currentTarget.value;
    const url = new URL(window.location.href);
    (value)
      ? url.searchParams.set('q', value)
      : url.searchParams.delete('q');
    debounce = setTimeout(() => window.history.pushState({}, '', url), 1000);
  }
}

function sync() {
  const params = new URLSearchParams(window.location.search);
  query = params.get('q') || '';
}

const require = (path: string) => import(/* @vite-ignore */path);

$effect(() => {
  sync();
  require('/pagefind/pagefind.js').then(pf => client = pf);
});
</script>


<svelte:window on:popstate={sync}/>

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
  <input class="c-search__input" placeholder="Start typing here!"
    bind:value={query}
    on:input={handler()}/>

  {#if query && result}
    {#await result}
      Loading...
    {:then {results}}
      <section class="c-search__results">
        <div>Showing results for "{query}" ({results.length})</div>
        {#each results as page (page.id)}
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
