import { getCollection } from "astro:content";


export async function getAllTags() {
  return (await Promise.all([
      getCollection('posts'),
      getCollection('slides'),
    ]))
    .flat()
    .reduce(
      (acc, next) => (next.data.tags?.forEach(tag => acc.add(tag)), acc),
      new Set<string>()
    );
}
