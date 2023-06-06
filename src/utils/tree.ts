import { Maybe } from "purify-ts";


interface Page {
  slug: string;
  data: { title: string };
}

export interface Tree {
  title: string;
  slug: Maybe<string>;
  children: Maybe<{ [key: string]: Tree }>;
}

export function collapse(pages: Page[]): Tree {
  const root: Tree = { title: '', slug: Maybe.empty(), children: Maybe.empty() };

  for (const page of pages) {
    const ptr = page.slug.split('/')
      .reduce((ptr, slug) => {
        // acquire pointer on next node in tree
        const next = ptr.children
          .chainNullable(trie => trie[slug])
          .orDefaultLazy(() => ({ title: slug, slug: Maybe.empty(), children: Maybe.empty() }));

        // update tree refs
        ptr.children = ptr.children
          .ifJust(trie => trie[slug] = next)
          .altLazy(() => Maybe.of({[slug]: next}));
        
        return next;
      }, root);
    
    ptr.slug = Maybe.of(`/${page.slug}/`);
    ptr.title = page.data.title;
  }

  return root;
}

export function pathify(...slugs: string[]): string {
  const path = slugs.map(part => part.trim().replace(/(^[\/]*|[\/]*$)/g, '')).join('/');
  return `/${path}/`;
}
