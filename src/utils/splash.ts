/** Subtitles dusplayed under site title in navbar. */
const subtitles = [
  "Playtime's over, playtime's now",
  "永遠のオデッセイ",
  "0% code coverage",
  "Type Driven Development",
  "Free as in Freedom",
  "Transmitting since 2021",
  "Approaching the speed of light",
  "Your Ad Here",
  "Here be dragons",
  "Kam was here",
  "あややや〜",
  "Haskell is nice 😳",
  "Coding my life away",
  "Powered by neural networks",
  "Volle Kraft voraus!",
  "In a quantum superposition",
  "IPA: [ka̠mo̞ɕi]",
  "遥か彼方を巡り廻る",
  "Disunified Field Theory of Magic",
  "Sp³ hybridized",
  "Lorem ipsum dolor sit amet",
  "Beautifully Imperfect and Hazy",
  "Destination unknown",
  "Now on Google's 3rd page 🎉",
  "A product of boredom",
  "Rotating multiaxially in R⁵",
  "Scientifically unproven",
  "Lost in translation",
  "Implemented in prolog and YAML",
  "Computer-aided diary thing",
  "Integration under the moon",
  "Spinning in retrograde",
  "Steady as she goes",
  "Слава України!",
  "2 year anniversary",
  "Because Twitter is passé",
  "{{ navbar.subtitle.text }}",
  "You're looking at it!",
  "a2Ftb3NoaS5vcmc=",
  ":: Thought a => a -> [String]",
  "Catch me if you can!",
  "In the eye of the beholder",
  "Another syncthing relay",
  "Now runs on NixOS!",
  "Arch Linux is easy",
];

const chance = Math.round(1 / (subtitles.length + 1) * 10000) / 100;
subtitles.push(`${chance}% chance for this message`);

export function bindSubtitle() {
  const target = document.getElementById("p-nav-splash");
  const choice = subtitles[Math.floor(Math.random() * subtitles.length)];
  target && (target.innerText = choice);
}
