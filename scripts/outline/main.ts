const HEADINGS = "h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]";

type Heading = HTMLHeadingElement;
type Link = HTMLLinkElement;

function findID(headings: Heading[]) {
  const scrollY = window.scrollY || window.pageYOffset;
  const midpoint = scrollY + window.innerHeight / 2;

  for (let i = 0; i < headings.length; i++) {
    const heading = headings[i];
    const next = headings[i + 1];

    const headingTop = heading.offsetTop;

    // If there's a next heading, we check if we're between this one and the next
    if (!next || next.offsetTop > midpoint) {
      if (headingTop <= midpoint) {
        return heading.id;
      }
      break;
    }
  }
}

document.addEventListener("DOMContentLoaded", () => {
  const headings = [...document.querySelectorAll(HEADINGS)] as Heading[];
  const outline = new Map(
    headings.map((h) => [
      h.id,
      document.querySelector(`.outline a[href="#${h.id}"]`) as Link | null,
    ]),
  );

  function onScroll() {
    const found = findID(headings);

    for (const [id, link] of outline.entries()) {
      if (found === id) {
        link?.classList.add("active");
      } else {
        link?.classList.remove("active");
      }
    }
  }

  let ready = true;
  function onScrollThrottled() {
    if (ready) {
      window.requestAnimationFrame(() => {
        onScroll();
        ready = true;
      });
      ready = false;
    }
  }

  document.addEventListener("scroll", onScrollThrottled, { passive: true });
  window.addEventListener("resize", onScrollThrottled);
  window.addEventListener("load", onScroll);
});
