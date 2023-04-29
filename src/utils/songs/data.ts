import circles from "../../data/circles.json";
import type { CollectionEntry } from "astro:content";


type Song = CollectionEntry<'songs'>;

interface Metadata {
  [key: string]: {
    /** Circle name */
    circle: string,
    /** Album title */
    title: string,
    /** Path to album cover image */
    cover: string,
  }
}


export function getAllCats(songs: Song[]): Set<string> {
  return songs.reduce(
    (cats, next) => (
      Object.keys(next.data.album).forEach(cat => cats.add(cat)),
      cats
    ),
    new Set<string>()
  );
}

function createMetadata(circles: CirclesSchema): Metadata {
  const metadata: Metadata = {};

  for (const circle of Object.keys(circles)) {
    const data = circles[circle];
    for (const cat of Object.keys(data.albums))
      metadata[cat] = { circle, ...data.albums[cat] }
  }
  return metadata;
}


export const ALBUMS: Metadata = createMetadata(circles);
