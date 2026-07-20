import { beforeAll, describe, expect, it } from "vitest";
import { parseUiModel } from "../recipe-editor/model";
import {
  emitBindings,
  emitFormula,
  emitOperation,
  emitResource,
  emitYield,
  formatNumber,
  formatQuantity,
  formatRange,
  sanitizeString,
  symbolize,
} from "./emit";
import { loadParser } from "./test-support";

beforeAll(loadParser);

/** Wrap a declaration in a minimal recipe and check the real parser accepts it. */
function parses(declaration: string): ReturnType<typeof parseUiModel> {
  return parseUiModel(`culinator 0.3;\n\nrecipe demo {\n    title "Demo";\n${declaration}\n}\n`);
}

describe("sanitizeString", () => {
  it("substitutes quotes rather than escaping them", () => {
    // The lexers consume `\"` but never unescape it, so a backslash written
    // here would survive into the rendered title.
    expect(sanitizeString('Ben "Chef" Ito')).toBe("Ben 'Chef' Ito");
    expect(sanitizeString('Ben "Chef" Ito')).not.toContain("\\");
  });

  it("collapses whitespace and trims", () => {
    expect(sanitizeString("  a\n\tb   c ")).toBe("a b c");
  });

  it("strips a trailing backslash that would swallow the closing quote", () => {
    expect(sanitizeString("path\\")).toBe("path");
    expect(
      parses(`    ingredient x { name "${sanitizeString("path\\")}"; }`).diagnostics,
    ).toHaveLength(0);
  });

  it("keeps a quoted value parseable end to end", () => {
    const model = parses(`    ingredient x { name "${sanitizeString('a "b" c')}"; }`);
    expect(model.diagnostics).toHaveLength(0);
    expect(model.resources[0].name).toBe("a 'b' c");
  });
});

describe("symbolize", () => {
  it("lowercases and joins on non-alphanumerics", () => {
    expect(symbolize("Bread Flour")).toBe("bread_flour");
    expect(symbolize("extra-virgin olive oil")).toBe("extra_virgin_olive_oil");
    expect(symbolize("  padded  ")).toBe("padded");
  });

  it("uniques against symbols already in the document", () => {
    const taken = new Set(["flour", "flour_2"]);
    expect(symbolize("Flour", taken)).toBe("flour_3");
  });

  it("avoids reserved words that would reparse as a declaration", () => {
    expect(symbolize("Yield")).toBe("yield_1");
    expect(symbolize("process")).toBe("process_1");
    // And the guarded symbol really does still parse as an ingredient.
    const model = parses(`    ingredient ${symbolize("Yield")} measured by mass { quantity 1 g; }`);
    expect(model.diagnostics).toHaveLength(0);
    expect(model.resources.map((resource) => resource.kind)).toEqual(["ingredient"]);
  });

  it("falls back when a name has no ASCII alphanumerics", () => {
    expect(symbolize("🥕")).toBe("item");
    expect(symbolize("焼き")).toBe("item");
    expect(symbolize("🥕", new Set(["item"]))).toBe("item_2");
    // A fallback that is itself a declaration keyword still gets guarded.
    expect(symbolize("🥕", new Set(), "ingredient")).toBe("ingredient_1");
  });

  it("does not start an identifier with a digit", () => {
    expect(symbolize("00 flour", new Set(), "ingredient")).toBe("ingredient_00_flour");
    expect(parses(`    ingredient ${symbolize("00 flour")} { }`).diagnostics).toHaveLength(0);
  });
});

describe("number and quantity formatting", () => {
  it("drops trailing zeros and float noise", () => {
    expect(formatNumber(500)).toBe("500");
    expect(formatNumber(0.1 + 0.2)).toBe("0.3");
    expect(formatNumber(0.25)).toBe("0.25");
  });

  it("formats quantities and ranges the way the grammar reads them", () => {
    expect(formatQuantity(400, "g")).toBe("400 g");
    expect(formatQuantity(2)).toBe("2");
    expect(formatRange(2, 3, "clove")).toBe("2 to 3 clove");
    expect(
      parses(`    ingredient garlic { quantity ${formatRange(2, 3, "clove")}; }`).diagnostics,
    ).toHaveLength(0);
  });
});

describe("emitBindings", () => {
  it("splits unquantified targets into a list and quantified ones into singles", () => {
    // The grammar's list form carries no amount, so these cannot be merged.
    expect(
      emitBindings("input", [
        { symbol: "macaroni" },
        { symbol: "water" },
        { symbol: "salt", quantity: "1 tbsp" },
      ]),
    ).toEqual(["input [macaroni, water];", "input salt 1 tbsp;"]);
  });

  it("emits nothing for no bindings", () => {
    expect(emitBindings("input", [])).toEqual([]);
  });

  it("produces bindings the parser reads back identically", () => {
    const lines = emitBindings("input", [{ symbol: "flour" }, { symbol: "salt", quantity: "5 g" }]);
    const model = parses(
      `    ingredient flour { }\n    ingredient salt { }\n    process p {\n        operation mix does mix {\n            ${lines.join("\n            ")}\n        }\n    }`,
    );
    expect(model.diagnostics).toHaveLength(0);
    const bindings = model.operations[0].inputBindings;
    expect(bindings).toEqual([{ symbol: "flour" }, { symbol: "salt", quantity: "5 g" }]);
  });
});

describe("emitResource", () => {
  it("round-trips through the parser", () => {
    const declaration = emitResource({
      kind: "ingredient",
      symbol: "bread_flour",
      measurement: "mass",
      name: "bread flour",
      quantity: "400 g",
      optional: true,
      notes: ["sifted"],
    });
    const model = parses(declaration);
    expect(model.diagnostics).toHaveLength(0);
    const resource = model.resources[0];
    expect(resource.symbol).toBe("bread_flour");
    expect(resource.name).toBe("bread flour");
    expect(resource.quantity).toBe("400 g");
    expect(resource.measurement).toBe("mass");
    expect(resource.optional).toBe(true);
    expect(resource.notes).toEqual(["sifted"]);
  });

  it("omits a declaration-level quantity for a divided ingredient", () => {
    // A divided ingredient's amounts live on the step bindings; declaring one
    // here as well double counts it.
    const declaration = emitResource({
      kind: "ingredient",
      symbol: "salt",
      measurement: "volume",
      divided: true,
      quantity: "1 tbsp",
    });
    expect(declaration).not.toContain("quantity");
    const resource = parses(declaration).resources[0];
    expect(resource.divided).toBe(true);
    expect(resource.quantity).toBeUndefined();
    expect(resource.measurement).toBe("volume");
  });

  it("emits an empty block when there is nothing to say", () => {
    expect(emitResource({ kind: "material", symbol: "dough", measurement: "mass" })).toBe(
      "    material dough measured by mass { }",
    );
    expect(parses(emitResource({ kind: "material", symbol: "dough" })).diagnostics).toHaveLength(0);
  });
});

describe("emitOperation", () => {
  it("round-trips a fully specified step", () => {
    const declaration = `    process p {\n${emitOperation({
      symbol: "make_roux",
      action: "heat",
      inputs: [{ symbol: "butter", quantity: "3 tbsp" }],
      produces: "roux",
      after: ["preheat"],
      duration: "5 min",
      labor: "active",
      heat: "medium_high",
      temperature: "350 fahrenheit",
      notes: ["whisk constantly"],
    })}\n    }`;
    const model = parses(
      `    ingredient butter { }\n    process q { operation preheat does heat { duration 1 min; } }\n${declaration}`,
    );
    expect(model.diagnostics).toHaveLength(0);
    const operation = model.operations.find((item) => item.symbol === "make_roux");
    expect(operation?.action).toBe("heat");
    expect(operation?.produces).toBe("roux");
    expect(operation?.after).toEqual(["preheat"]);
    expect(operation?.durationMinutes).toBe(5);
    expect(operation?.labor).toBe("active");
    expect(operation?.heatLevel).toBe("medium_high");
    expect(operation?.targetTemperature).toBe("350 fahrenheit");
    expect(operation?.notes).toEqual(["whisk constantly"]);
  });
});

describe("emitYield and emitFormula", () => {
  it("emits a yield the parser accepts", () => {
    const model = parses(
      emitYield({ symbol: "servings", measurement: "count", amount: "4 count" }),
    );
    expect(model.diagnostics).toHaveLength(0);
  });

  it("emits a baker's formula the parser accepts", () => {
    const model = parses(
      emitFormula({
        symbol: "dough",
        basis: "as BakersFormula",
        target: "1800 g",
        ingredients: [
          { symbol: "bread_flour", type: "Flour<BakersPercent>", stage: "final", baker: "80%" },
          { symbol: "water", type: "Liquid<BakersPercent>", stage: "final", baker: "70%" },
        ],
      }),
    );
    expect(model.diagnostics).toHaveLength(0);
  });
});
