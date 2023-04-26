import { defineMarkdocConfig } from '@astrojs/markdoc/config';
import Lyrics from './src/components/markdown/lyrics/Lyrics.astro';


export default defineMarkdocConfig({
  tags: {
    lyrics: {
      render: Lyrics,
      attributes: {
        type: { type: String },
      }
    }
  }
})
