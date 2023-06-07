type TODO = any;

/** @file src/data/circles.json */
interface CirclesSchema {
  /** slug */
  [key: string]: {
    name: string,
    albums: {
      /** catalog number */
      [key: string]: {
        title: string;
        cover: string;
      }
    }
  }
}
