import { defineCollection, z } from 'astro:content';


export const collections = {
  posts: defineCollection({
    schema: z.object({
      title: z.string(),
      date: z.date(),
      tags: z.array(z.string()).optional(),
    })
  }),
  slides: defineCollection({
    schema: z.object({
      title: z.string(),
      date: z.date(),
      tags: z.array(z.string()).optional(),
    })
  }),
  songs: defineCollection({
    schema: z.object({
      title: z.string(),
      composer: z.string().optional(),
      lyrics: z.string().optional(),
      origin: z.array(z.string()).optional(),
      album: z.record(
        z.string(),
        z.object({
          track: z.number(),
          vocal: z.array(z.string()).optional()
        })
      )
    })
  })
}
