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


<article class="search">
  <h1>Search</h1>

  <input class="search-input" bind:value on:input={search} placeholder="Start typing here!"/>

  {#if value}
    <section class="results">
      <div>Showing results for "{value}" ({results.length})</div>
      {#each results as result}
        {@const meta = metadata[result.ref]}
        {@const date = dayjs(meta.date)}
        <a class="result" href={result.ref}>
          <span class="name">{meta.title}</span>
          <time class="date" datetime={date.toISOString()}>{date.format("MMM DD, YYYY")}</time>
        </a>
      {/each}
    </section>
  {:else}
    <div>No results to show yet...</div>
  {/if}
</article>

<svelte:window on:popstate={sync} />


<style lang="scss">
  .search {
    margin: 2em auto;
    padding: 0 4em;
    max-width: 52em;

    .search-input {
      width: 100%;
      padding: 0.5em 1em;
      margin-bottom: 0.5em;
      box-sizing: border-box;
    }

    .results {
      display: grid;
      row-gap: 0.5em;
    }

    .result {
      display: flex;
      flex-direction: row;
      padding: 0.5em;
      background-color: white;
      box-shadow: rgba(0, 0, 0, 0.1) 0px 1px 3px 0px, rgba(0, 0, 0, 0.06) 0px 1px 2px 0px;
      transition: box-shadow linear 100ms;
      text-decoration: none;

      .date {
        margin-left: auto;
        color: rgb(65, 65, 65);
      }

      &:focus-within,
      &:hover {
        box-shadow: rgba(0, 0, 0, 0.1) 0px 4px 6px -1px, rgba(0, 0, 0, 0.06) 0px 2px 4px -1px;
      }
    }
  }
</style>
