import { createSignal, onMount } from "solid-js";
import dayjs from "dayjs";
import lunr from "lunr";


const KEY = 'q' as const;
export default function Search() {
  const [query, setQuery] = createSignal('');

  // Data loaded from server
  let index: lunr.Index;
  let metadata: any;
  async function load() {
    const data = await fetch('/search.json').then(r => r.json());
    index = lunr.Index.load(data.index);
    metadata = data.metadata;
  }

  // Dynamically calculated from query
  const results = () => index?.search(query()) ?? [];

  // Update URL query and history
  let debounce: number;
  function syncHistory(value: string) {
    clearTimeout(debounce);
    debounce = setTimeout(() => {
      const url = new URL(window.location.href);
      (value) ? url.searchParams.set(KEY, value) : url.searchParams.delete(KEY);
      window.history.pushState({}, '', url);
    }, 1000);
  }

  // set query to URL param
  function sync() {
    clearTimeout(debounce);
    setQuery(new URLSearchParams(window.location.search).get('q') || '');
  }

  function onInput(value: string) {
    setQuery(value);
    syncHistory(value);
  }

  onMount(async () => {
    await load().then(sync);
    window.addEventListener('popstate', sync);
  });

  return (
    <article class="c-search">
      <h1>Search</h1>
      <input class="c-search__input" placeholder="Start typing here!"
        value={query()}
        onInput={e => onInput(e.target.value)}/>

      {query() && (
        <section class="c-search__results">
          <div>Showing results for "{query()}" ({results().length})</div>
          {results().map(result => {
            const meta = metadata[result.ref];
            const date = dayjs(meta.date);
            return (
              <a class="c-search__result" href={result.ref}>
                <span class="c-search__name">{meta.title}</span>
                <time class="c-search__date" datetime={date.toISOString()}>{date.format("MMM DD, YYYY")}</time>
              </a>
            )
          })}
        </section>
      ) || (
        <div>No results to show yet...</div>
      )}
    </article>
  );
}
