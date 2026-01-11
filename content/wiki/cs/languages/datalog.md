---
title: Datalog
---

Essentially a subset of Prolog.

Prolog allows "function symbols", which means you can generate infinite new
data, so a query might never stop running. Datalog bans function symbols and
only allows variables and constants, because of this, the set of all possible
answers is limited to the facts existing in your database, or derived.

Calling it a "superset" of SQL is mostly true regarding logic, but false
regarding features. Datalog handles Recursion naturally (e.g., finding all
ancestors). Standard SQL strictly cannot do this. You need specific extensions
like "Recursive CTEs" (`WITH RECURSIVE`) to make SQL behave like Datalog.

- SQL counts duplicates (a table can have two identical rows). Datalog usually
  treats data as Sets (no duplicates).
- SQL has `SUM`, `AVG`, `COUNT`. Pure Datalog does not (though many practical
  implementations add this).
- SQL has 3-valued logic (`True`, `False`, `Null`). Datalog is strictly 2-valued.

| Feature         | Standard SQL             | Datalog             | Prolog       |
| :-------------- | :----------------------- | :------------------ | :----------- |
| **Core logic**  | Relational algebra       | Horn clauses        | Horn clauses |
| **Recursion**   | No (requires extensions) | Yes                 | Yes          |
| **Termination** | Guaranteed               | Guaranteed          | No           |
| **Duplicates**  | Yes (bag)                | No (set)            | Yes          |
| **Complexity**  | $AC^0$ (data)            | $P$-Complete (data) | Undecidable  |

- <https://pages.cs.wisc.edu/~paris/cs784-s17/lectures/>
