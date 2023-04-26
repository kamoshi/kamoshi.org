import type { CollectionEntry } from "astro:content";


type Song = CollectionEntry<'songs'>;


export function getAllCats(songs: Song[]): Set<string> {
  return songs.reduce(
    (cats, next) => (
      Object.keys(next.data.album).forEach(cat => cats.add(cat)),
      cats
    ),
    new Set<string>()
  );
}
