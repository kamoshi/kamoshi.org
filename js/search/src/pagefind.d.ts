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
