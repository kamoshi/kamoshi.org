const HEADINGS = 'h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]';

type Heading = HTMLHeadingElement;
type Link = HTMLAnchorElement;

function findID(headings: Heading[]) {
  const scrollY = globalThis.scrollY;
  // Set the trigger line near the top of the viewport. 100px is a standard
  // buffer, adjust this if you have a sticky header
  const triggerLine = scrollY + 100;

  // Default to the first heading so something is always highlighted at the very top
  let activeId = headings[0]?.id;

  for (let i = 0; i < headings.length; i++) {
    const heading = headings[i];

    // If this heading has scrolled past our top trigger line, it's the active one (so far)
    if (heading.offsetTop <= triggerLine) {
      activeId = heading.id;
    } else {
      // Because headings are ordered vertically, as soon as we find one
      // BELOW the trigger line, we can stop looking.
      break;
    }
  }

  return activeId;
}

globalThis.document.addEventListener('DOMContentLoaded', () => {
  const headings = [
    ...globalThis.document.querySelectorAll(HEADINGS),
  ] as Heading[];
  const outline = new Map(
    headings.map((h) => [
      h.id,
      globalThis.document.querySelector(
        `.outline a[href="#${h.id}"]`,
      ) as Link | null,
    ]),
  );

  function onScroll() {
    const found = findID(headings);

    for (const [id, link] of outline.entries()) {
      if (found === id) {
        link?.classList.add('active');
      } else {
        link?.classList.remove('active');
      }
    }
  }

  let ready = true;
  function onScrollThrottled() {
    if (ready) {
      globalThis.requestAnimationFrame(() => {
        onScroll();
        ready = true;
      });
      ready = false;
    }
  }

  globalThis.document.addEventListener('scroll', onScrollThrottled, {
    passive: true,
  });
  globalThis.addEventListener('resize', onScrollThrottled);
  globalThis.addEventListener('load', onScroll);
});
