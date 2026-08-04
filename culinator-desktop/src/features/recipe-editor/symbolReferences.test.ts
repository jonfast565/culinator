import { describe, expect, it } from "vitest";
import type { UiRecipeModel } from "./model";
import {
  bindingForCursor,
  buildSymbolReferenceIndex,
  definedSymbolSpans,
  findSymbolSpans,
  identifierAt,
  isInsideStringOrComment,
} from "./symbolReferences";

const SOURCE = `culinator 0.3;
recipe demo {
    title "Demo";
    ingredient flour measured by mass {
        quantity 500 g;
        substitutes [meal];
    }
    ingredient meal measured by mass {
        quantity 100 g;
    }
    process dough {
        operation mix does mix {
            input flour;
            input meal;
            produces dough_mix;
            duration 5 min;
            labor active;
        }
        operation knead does knead {
            input dough_mix;
            after mix;
            duration 10 min;
            labor active;
        }
    }
}
`;

function model(): UiRecipeModel {
  const flourStart = SOURCE.indexOf("ingredient flour");
  const mealStart = SOURCE.indexOf("ingredient meal");
  const mixStart = SOURCE.indexOf("operation mix");
  const kneadStart = SOURCE.indexOf("operation knead");
  const flourEnd = SOURCE.indexOf("}", flourStart) + 1;
  const mealEnd = SOURCE.indexOf("}", mealStart) + 1;
  const mixEnd = SOURCE.indexOf("}", mixStart) + 1;
  const kneadEnd = SOURCE.indexOf("}", kneadStart) + 1;

  return {
    title: "Demo",
    symbol: "demo",
    resources: [
      {
        symbol: "flour",
        name: "flour",
        kind: "ingredient",
        measurement: "mass",
        substitutes: ["meal"],
        range: { start: flourStart, end: flourEnd },
      },
      {
        symbol: "meal",
        name: "meal",
        kind: "ingredient",
        measurement: "mass",
        range: { start: mealStart, end: mealEnd },
      },
      {
        symbol: "dough_mix",
        name: "dough mix",
        kind: "material",
        measurement: "unspecified",
      },
    ],
    processes: [{ symbol: "dough" }],
    operations: [
      {
        symbol: "mix",
        action: "mix",
        process: "dough",
        durationMinutes: 5,
        labor: "active",
        after: [],
        inputs: ["flour", "meal"],
        inputBindings: [{ symbol: "flour" }, { symbol: "meal" }],
        equipment: [],
        produces: "dough_mix",
        range: { start: mixStart, end: mixEnd },
      },
      {
        symbol: "knead",
        action: "knead",
        process: "dough",
        durationMinutes: 10,
        labor: "active",
        after: ["mix"],
        inputs: ["dough_mix"],
        inputBindings: [{ symbol: "dough_mix" }],
        equipment: [],
        range: { start: kneadStart, end: kneadEnd },
      },
    ],
    yields: [],
    diagnostics: [],
  };
}

describe("findSymbolSpans", () => {
  it("finds whole-word occurrences", () => {
    const spans = findSymbolSpans(SOURCE, "flour");
    expect(spans.length).toBeGreaterThanOrEqual(2);
    expect(SOURCE.slice(spans[0].start, spans[0].end)).toBe("flour");
  });
});

describe("buildSymbolReferenceIndex", () => {
  it("records definitions and cross-operation uses", () => {
    const index = buildSymbolReferenceIndex(SOURCE, model());
    const flour = index.get("flour");
    expect(flour?.definition).toBeTruthy();
    expect(flour?.uses.length).toBeGreaterThanOrEqual(1);
    expect(SOURCE.slice(flour!.uses[0].start, flour!.uses[0].end)).toBe("flour");

    const mix = index.get("mix");
    expect(mix?.definition).toBeTruthy();
    expect(mix?.uses.some((use) => SOURCE.slice(use.start, use.end) === "mix")).toBe(true);

    const meal = index.get("meal");
    expect(meal?.uses.length).toBeGreaterThanOrEqual(2);
  });
});

describe("bindingForCursor", () => {
  it("resolves a use site back to the binding", () => {
    const index = buildSymbolReferenceIndex(SOURCE, model());
    const use = index.get("flour")!.uses[0];
    const binding = bindingForCursor(index, SOURCE, use.start + 1);
    expect(binding?.symbol).toBe("flour");
  });

  it("reads the identifier under the cursor", () => {
    const pos = SOURCE.indexOf("input flour") + "input ".length;
    expect(identifierAt(SOURCE, pos)).toBe("flour");
  });
});

describe("definedSymbolSpans", () => {
  it("colors every occurrence of recipe-defined names", () => {
    const index = buildSymbolReferenceIndex(SOURCE, model());
    const spans = definedSymbolSpans(SOURCE, index);
    const flour = spans.filter((span) => SOURCE.slice(span.start, span.end) === "flour");
    expect(flour.length).toBeGreaterThanOrEqual(2);
    expect(flour.every((span) => span.kind === "resource")).toBe(true);
  });

  it("skips matches inside strings and comments", () => {
    const source = `ingredient flour {};\n// flour\nnote "flour";\ninput flour;`;
    const index = new Map([
      [
        "flour",
        {
          symbol: "flour",
          kind: "resource" as const,
          definition: { start: 11, end: 16 },
          uses: [],
        },
      ],
    ]);
    const spans = definedSymbolSpans(source, index);
    expect(spans.map((span) => source.slice(span.start, span.end))).toEqual(["flour", "flour"]);
    expect(isInsideStringOrComment(source, source.indexOf("// flour") + 3)).toBe(true);
    expect(isInsideStringOrComment(source, source.indexOf('"flour"') + 1)).toBe(true);
  });
});
