#set page(paper: "a4", margin: (x: 2cm, y: 2.5cm))
#set text(font: "Times New Roman", size: 11pt, lang: "pl")
#set par(justify: true)

#align(center)[
  #text(size: 22pt)[Algebra liniowa 1]\
  #v(2mm)
  #text(size: 16pt)[I kolokwium, grudzień '18]
]

#v(4mm)

Na pierwszej stronie pracy proszę napisać nazwę kursu, z którego odbywa się kolokwium, swoje imię i nazwisko, numer indeksu, wydział, kierunek, rok, datę oraz sporządzić poniższą tabelkę. Ponadto proszę ponumerować i podpisać wszystkie kartki pracy.

#v(2mm)

#align(center)[
  #table(
    columns: (2cm, 2cm, 2cm, 2cm, 4cm),
    align: center + horizon,
    stroke: 0.5pt,
    table.cell(rowspan: 2)[#text(size: 36pt, weight: "bold", style: "italic")[B]],
    [*1*], [*2*], [*3*], [*suma*],
    [#v(0.6cm)], [], [], []
  )
]

#v(2mm)

Treści zadań proszę nie przepisywać. *Rozwiązanie zadania o numerze $n$ należy napisać na $n$-tej kartce pracy.* Na rozwiązanie zadań przeznaczono 45 minut, za rozwiązanie każdego zadania można otrzymać od 0 do 5 punktów. W rozwiązaniach proszę: formułować lub nazywać wykorzystywane twierdzenia, przytaczać stosowane wzory, uzasadniać wyciągane wnioski, starannie sporządzać rysunki. Powodzenia !

#align(right)[
  Bogdan Pawlik
]

#v(2mm)

#align(center)[
  #text(size: 16pt, weight: "bold")[ZADANIA]
]

#rect(width: 100%, stroke: 0.5pt, inset: 1.5em)[
  #grid(
    columns: (auto, auto, 1fr),
    gutter: 1.5em,
    align: (top, top, horizon),
    [1)],
    [
      $ A = mat(
        0, 0, ⋯, 0, 1;
        0, 0, ⋯, 1, 0;
        ⋮, ⋮, ⋱, ⋮, ⋮;
        0, 1, ⋯, 0, 0;
        1, 0, ⋯, 0, 0
      ) $
    ],
    [Znaleźć wartość $det(A)$ w zależności od wymiaru macierzy A.]
  )

  #v(1.5em)

  #grid(
    columns: (auto, 1fr),
    gutter: 1em,
    [2)],
    [Znaleźć układ 8 równań z czterema niewiadomymi $(x,y,z,t)$, którego rozwiązaniem jest $x,y,z$ – dowolne, $t$ – takie że $t + x = 77$.]
  )

  #v(1.5em)

  #grid(
    columns: (auto, 1fr),
    gutter: 1em,
    align: horizon,
    [3)],
    [Obliczyć wartość wyrażenia #h(1em) $ ((sqrt(2)i - sqrt(2)) / (i sqrt(3) - 1))^(2345) $ #h(1em) (wynik podać w postaci algebraicznej).]
  )
]
