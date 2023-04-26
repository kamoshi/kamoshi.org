/** @file src/data/circles.json */
interface CirclesSchema {
  /** Circle name */
  [key: string]: {
    albums: {
      /** Catalog number */
      [key: string]: {
        title: string;
      }
    }
  }
}
