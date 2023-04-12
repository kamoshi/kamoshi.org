<script lang="ts">
  import dayjs from "dayjs";
  import lunr from "lunr";
  import { onMount } from "svelte";

  const KEY = 'q' as const;
  let value = '';
  let index: lunr.Index;
  let results: lunr.Index.Result[] = [];
  let metadata: any;

  /** Update the value so that it reflects the URL */
  function sync() {
    value = new URLSearchParams(window.location.search).get('q') || '';
    search();
  }

  let debounce: number; // timer handle
  /** Update browser URL with the query string */
  function syncHistory(value: string) {
    clearTimeout(debounce);
    debounce = setTimeout(() => {
      const url = new URL(window.location.href);
      (value)
        ? url.searchParams.set(KEY, value)
        : url.searchParams.delete(KEY);
      window.history.pushState({}, '', url);
    }, 1000);
  }

  async function load() {
    const data = await fetch('/search.json').then(r => r.json());
    index = lunr.Index.load(data.index);
    metadata = data.metadata;
  }

  function search() {
    results = (!value || !index) ? [] : index.search(value);
    syncHistory(value);
  }

  onMount(() => load().then(sync).then(search));
</script>


<article class="c-search">
  <h1>Search</h1>

  <input class="c-search__input" bind:value on:input={search} placeholder="Start typing here!"/>

  {#if value}
    <section class="c-search__results">
      <div>Showing results for "{value}" ({results.length})</div>
      {#each results as result}
        {@const meta = metadata[result.ref]}
        {@const date = dayjs(meta.date)}
        <a class="c-search__result" href={result.ref}>
          <span class="c-search__name">{meta.title}</span>
          <time class="c-search__date" datetime={date.toISOString()}>{date.format("MMM DD, YYYY")}</time>
        </a>
      {/each}
    </section>
  {:else}
    <div>No results to show yet...</div>
  {/if}
</article>

<svelte:window on:popstate={sync} />
