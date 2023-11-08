import { defineCollection, z } from 'astro:content';


export const collections = {
  posts: defineCollection({
    schema: z.object({
      title: z.string(),
      date: z.date(),
      icon: z.string().optional(),
      desc: z.string().optional(),
    })
  }),
  slides: defineCollection({
    schema: z.object({
      title: z.string(),
      date: z.date(),
      tags: z.array(z.string()).optional(),
    })
  }),
  wiki: defineCollection({
    schema: z.object({
      title: z.string(),
    })
  })
}
