import lunr from 'lunr';
import { getCollection } from 'astro:content';
import type { APIContext } from "astro";


const posts = await Promise.all([
    getCollection('posts').then(a => a.map(p => ({...p, slug: `/posts/${p.slug}/`}))),
    getCollection('aoc').then(a => a.map(p => ({...p, slug: `/aoc/${p.slug}/`}))),
    getCollection('slides').then(a => a.map(p => ({...p, slug: `/slides/${p.slug}/`}))),
  ])
  .then(array => array.flat());

const metadata = posts.reduce(
  (acc, next) => (
    acc[next.slug] = { title: next.data.title, date: next.data.date },
    acc
  ),
  {} as {[key: string]: {title: string, date: Date}}
)

const index = lunr(function() {
  this.ref('slug');
  this.field('body');

  for (const post of posts)
    this.add(post);
})

export async function get(_: APIContext) {
  return {body: JSON.stringify({ index, metadata })};
}
