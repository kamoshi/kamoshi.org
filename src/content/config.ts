import { defineCollection, z } from 'astro:content';


const post = z.object({
  title: z.string(),
  date: z.date(),
  icon: z.string().optional(),
  desc: z.string().optional(),
})

export const collections = {
  posts: defineCollection({ schema: post }),
  slides: defineCollection({
    schema: z.object({
      title: z.string(),
      date: z.date(),
      tags: z.array(z.string()).optional(),
      animate: z.boolean().optional(),
    })
  }),
  wiki: defineCollection({
    schema: z.object({
      title: z.string(),
    })
  })
}
