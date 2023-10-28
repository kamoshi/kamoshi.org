import { createResource, createSignal, For, onMount } from "solid-js";


interface Pagefind {
  search: (query: string) => Promise<PagefindResponse>;
}

interface PagefindResult {
  id: string;
  data: () => Promise<PagefindDocument>;
}

interface PagefindResponse {
  results: PagefindResult[];
}

interface PagefindDocument {
  url: string;
  excerpt: string;
  filters: {
    author: string;
  };
  meta: {
    title: string;
    image: string;
  };
  content: string;
  word_count: number;
}

const enum PagefindModule {
  LINK = '/pagefind/pagefind.js',
}

const loadPagefind = () => import(/* @vite-ignore */PagefindModule.LINK) as Promise<Pagefind>;


function Result(props: { page: PagefindResult }) {
  const [data, setData] = createSignal<PagefindDocument>();

  onMount(async () => await props.page.data().then(setData))

  return (
    <>
      {data() && (
        <a class="c-search__result" href={data()!.url}>
          <header class="c-search__header">
            <h2 class="c-search__title">{data()!.meta.title}</h2>
            {/* <time class="c-search__date" datetime={dayjs().toISOString()}>{dayjs().format("MMM DD, YYYY")}</time> */}
          </header>
          <div class="c-search__excerpt" innerHTML={data()!.excerpt}></div>
        </a>
      ) || (
        <div>Loading...</div>
      )}
    </>
  )
}

const KEY = 'q' as const;
export default function Search() {
  const [query, setQuery] = createSignal('');

  // Search
  let pagefind: Pagefind;
  const [pages] = createResource(query, async (query: string) => await pagefind.search(query));

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
    pagefind = await loadPagefind();
    sync();
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
          <div>Showing results for "{query()}" ({pages()?.results.length || 0})</div>
          {pages() && <For each={pages()!.results}>{(page, i) => (
            <Result page={page} />
          )}</For>}
        </section>
      ) || (
        <div>No results to show yet...</div>
      )}
    </article>
  );
}
