import { defineCollection, z } from 'astro:content';


export const collections = {
  aoc: defineCollection({
    schema: z.object({
      title: z.string(),
      date: z.date(),
      day: z.number(),
      stars: z.number(),
      math: z.boolean().optional()
    })
  }),
  slides: defineCollection({
    schema: z.object({
      title: z.string(),
      date: z.date(),
    })
  })
}
