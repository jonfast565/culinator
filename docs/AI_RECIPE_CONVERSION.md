# Converting recipes to Culinator DSL — AI conversion notes

Rules for translating a prose recipe (web page, photo/OCR, cookbook scan) into
Culinator 0.3 DSL without losing the nuance that prose carries implicitly.
Derived from auditing the three seed recipes against their originals
(Alton Brown's guacamole, baked mac & cheese, and crepes) and their photos.
Guidance for anyone (human or AI) authoring `.cg` from a source recipe; the AI
importer prompt (`culinator-import/src/openai.rs`) encodes the highlights.

The audit that produced this doc also drove the language: the gaps it found
were closed with first-class fields (`to_taste`, `size`, `variant`, `note`,
`repeat` — see [GRAMMAR.ebnf](./GRAMMAR.ebnf)), and the three seed recipes were
rewritten to use them. The "seed bug" call-outs below describe what the seeds
used to do wrong and now do right — keep them as worked examples.

Grammar reference: [GRAMMAR.ebnf](./GRAMMAR.ebnf). Importer plumbing:
[OCR_AI_IMPORT.md](./OCR_AI_IMPORT.md).

## 1. Ingredient-line prep descriptors are *work*, not adjectives

Prose ingredient lines smuggle in processing: "3 ripe Hass avocados, **halved,
pitted, and peeled**", "2 small Roma tomatoes, **seeded and diced**", "1 clove
garlic, **minced**". Decide per descriptor:

- **Condition you shop for / arrive at before the clock starts** → `state` on
  the ingredient (`state ripe;`, `state melted;`). No time cost, no operation.
- **Knife/prep work the cook performs** → a `prep` declaration (or explicit
  operation) with a realistic `duration`, so scheduling and labor accounting
  see it: `prep dice tomatoes into diced_tomatoes { duration 3 min; }`.
- **Compound descriptors decompose**: "seeded and diced" is *one* prep op is
  fine (`prep dice ...`) but the discarded part ("seeded") should at least
  survive in the op or ingredient via a `note "seeded before dicing";`
  (first-class, repeatable) — don't silently drop it.

Seed bug this rule comes from: the guacamole *used to* declare `state ripe`
avocados and then an operation that mashed them — but the original's "halved,
pitted, and peeled" prep vanished entirely. There is no free path from a whole
avocado to mashable pulp, so the seed now emits `prep pit avocados into
avocado_pulp { note "halve, pit, and peel"; duration 3 min; }` and `mash` takes
`avocado_pulp`.

Borderline case: `state grated` cheese (mac & cheese seed) is acceptable *only
if* you deliberately model it as procured pre-grated. If the source lists a
block of cheese the cook grates, emit `prep grate cheddar ...` instead. Pick
one interpretation and be consistent within a recipe.

## 2. Quantities measured *after* prep

"1 tablespoon **chopped** fresh cilantro" means the tablespoon is measured on
the chopped product, not the sprigs. When the source measures post-prep, put
the quantity on the ingredient as written but keep the prep op, and add
`note "measured after chopping";` so round-tripping doesn't re-read it as 1 tbsp
of whole leaves. Never convert it into a pre-prep quantity by guessing.

## 3. `divided` ingredients — never merge the splits

"1 tablespoon plus 1/2 teaspoon kosher salt, **divided**" or butter/cheese
used partly in the sauce and partly on the topping:

- Mark the ingredient `divided true;` and omit a single top-level `quantity`.
- Give each consuming operation its per-step amount via the **single**-binding
  form: `input butter 3 tbsp;`. The list form (`input [a, b];`) cannot carry a
  quantity — split the quantified binding onto its own `input` line.

Seed bugs this rule comes from (mac & cheese): the original splits butter
3 tbsp (roux) + 3 tbsp melted (tossed with panko) and Monterey jack 5 oz
(sauce) + 3 oz (sprinkled on top). The seed *used to* lump all butter into the
sauce and all jack into the stir-in — so the topping steps were literally wrong
(dry panko, no cheese crust). The seed now marks both `divided true;` and binds
`input jack 5 oz;` / `input jack 3 oz;`. Merging divided amounts changes the dish.

## 4. "Plus more", "to taste", "for the pan"

"1/4 tsp cayenne, **plus more to taste**", "butter, **plus additional for the
pan**". Capture the written base quantity, then mark the open-ended part:

- If the extra is a separate use (pan butter), model it as its own `optional
  true;` ingredient bound to the op that uses it, with a `note` for the use.
- If it's seasoning adjustment, keep the base quantity and add the first-class
  `to_taste true;` flag (sits alongside `optional`/`divided`).

Never round it away — the crepes seed *used to* drop the pan butter entirely
(now a `pan_butter` optional ingredient) and the guacamole turned "1/4 tsp plus
more" into "1 pinch" (now `quantity 0.25 tsp; to_taste true;`).

## 5. Every op the source performs exists, even the boring ones

Preheat oven, grease the dish, drain the pasta, cool 5 minutes before
serving. These carry real scheduling weight (a 350°F preheat is ~15 passive
minutes that overlaps prep) and real correctness weight (ungreased dish,
undrained pasta). Emit them:

```
operation preheat does heat {
    target oven;
    temperature 350 fahrenheit;
    duration estimated 15 min;
    labor automated;
}
```

`target` bindings point at equipment; `labor automated` / `passive` keeps the
concurrency view honest. The mac & cheese seed *used to* omit preheat, grease,
drain, season (salt + pepper were missing from its ingredient list entirely),
and the 5-minute cool; it now includes all of them. An AI conversion must not
summarize steps away.

## 6. Split operations where labor, heat, or doneness changes

The original roux sequence — melt butter (medium), whisk flour to "pale
blond" (~3 min), bloom spices (~1 min), whisk in dairy until "slightly
thickened" (7–8 min) — *used to* be one 10–12 min `make_sauce` op in the seed,
which lost three doneness cues and the heat change (medium → medium-high). The
seed now runs `melt_butter → make_roux → bloom_spices → build_base`, each with
its own heat/labor/`until`. Rule of thumb: start a new operation whenever any
of these changes:

- `heat` level or `temperature`
- `labor` class (active stirring vs. monitoring)
- an `until` cue is checked ("pale blond", "aromatic", "coats the spoon")

`until` is typed — use it: `until visual "pale blond";`,
`until internal_temp 74 celsius;`, `until texture "slightly thickened";`.

## 7. Output `state` must describe the actual result (photos are evidence)

The guacamole seed *used to* declare `material mashed_avocado { state smooth; }`
while the original says "mash … **leaving some larger chunks for texture**" —
and the source photo shows plainly chunky guacamole. When a photo of the
finished dish or step is available, use it to validate texture/consistency
claims. The seed is now `state chunky;` with the technique nuance in a `note`
on the `mash` op.

## 8. Equipment, containers, and environment are resources — declare them

The originals name a potato masher, a large mixing bowl, plastic wrap, a
4-quart casserole, a 4-quart pot, a 3-quart saucepan, a colander, a blender,
and a 10-inch nonstick pan. The seeds *used to* declare **zero** equipment;
they now declare and bind these. Declare what the source names and bind it
(`tool`, `container`, `equipment`, `target`) — this is what lets the scheduler
detect resource conflicts (you can't boil pasta and make the sauce in the same
pot, but the mac & cheese seed uses a separate `pot` and `saucepan` so those
overlap). Equipment sizes go on the resource in its `name` ("10-inch nonstick
pan"). Ambient conditions ("rest at room temperature") have no binding keyword
yet — carry them in a `note` (see §10).

Do not confuse equipment size with ingredient `size` (§9): the latter is a
first-class grade on a count-measured ingredient.

## 9. Yields, times, holding windows, and units

- **Yield ranges are expressible** — "10 to 12 crepes" is
  `amount 10 count to 12 count;`, not an invented average (the seed said 11;
  now fixed).
- **Ingredient `size` is first-class** — "2 small Roma tomatoes", "1/2 medium
  onion", "2 large eggs" carry `size "small"` / `"medium"` / `"large"` instead
  of smuggling the grade into `name`, which keeps the count clean for scaling.
- **Open-ended holds** use the grammar's `up to` form (`duration up to 30 day;`
  for "freeze up to 1 month"). A minimum-plus-ceiling like the crepe batter
  ("refrigerate 1 hour; keeps up to 48 hours") is `duration 1 hour;` with a
  `note` about the 48-hour ceiling — don't turn a 1-hour-minimum chill into an
  arbitrary "1 to 8 hours".
- **Match the measurement dimension to the unit.** `measured by mass` with
  `quantity 0.5 tsp` is contradictory (tsp classifies as volume in
  `Dimension::from_unit`). Spices measured in spoons are `measured by volume`;
  use mass only when the source gives grams/ounces. All three seeds *used to*
  get this wrong and are now fixed.
- Source "active time / total time" metadata can ride generic properties
  (`active_time 10 min;`, `total_time 130 min;`) until first-class support
  exists.

## 10. First-class fields for prose nuance (and the gaps that remain)

The seed audit drove new syntax. These now exist as typed fields:

| Concept | Syntax | Semantics |
| --- | --- | --- |
| **Batching / repetition** | `repeat 11;` on an op | `duration` is per-repetition; the scheduler counts `duration × repeat` (crepes: `duration 45 seconds to 90 seconds; repeat 11;` ≈ 16 min, not 90 s) |
| **"To taste" seasoning** | `to_taste true;` on an ingredient | A base `quantity` may still be declared; flags the open-ended adjustment |
| **Size grade** | `size "small";` on a count ingredient | Structured grade for scaling; keeps `name` clean |
| **Variant sets** | `variant "sweet";` on ingredients | Ingredients sharing a label are one alternative set; those without are always included |
| **Technique note** | `note "do not rinse";` on ops *and* ingredients | Repeatable free text preserved as a typed list |

Still needing design (use the workaround and emit an import `warning`):

| Concept | Example | Gap / workaround |
| --- | --- | --- |
| **Variant *exclusivity* enforcement** | sweet vs. savory crepes | `variant` labels the sets but nothing yet stops selecting both, and variant ingredients aren't bound to an op; treat as metadata |
| **Storage / make-ahead lifecycle** | "refrigerate several days or freeze up to 1 month; thaw on rack" | A trailing `optional` rest op with `duration up to …;` + a `note`; no true storage-state modeling (the crepes seed's `store` op does this) |
| **Ambient environment binding** | "rest at room temperature" | No `environment` binding keyword on operations yet; carry as a `note` |

## 11. Checklist for an AI conversion pass

1. Every ingredient-line descriptor is either a `state` or a prep op (§1).
2. No `divided` amounts merged; per-step **single-binding** quantities used (§3).
3. "Plus more / to taste / for the pan" captured via `to_taste` / an optional
   ingredient, not rounded away (§4).
4. Preheat, grease, drain, rest, cool steps all present (§5).
5. One op per (heat, labor, until) regime (§6).
6. Doneness cues verbatim in typed `until` fields (§6).
7. Output material states match the source text *and photos* (§7).
8. Named equipment declared and bound (§8).
9. Yield/durations as written — ranges stay ranges; `size` grades lifted off
   `name` (§9).
10. Unit dimension matches `measured by` (§9).
11. Batched steps use `repeat`; technique detail in `note`; variants labelled
    with `variant` (§10).
12. Anything still inexpressible → `note` + a `warnings` entry (§10).
