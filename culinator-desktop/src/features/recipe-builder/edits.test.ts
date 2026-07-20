import { beforeAll, describe, expect, it } from "vitest";
import { parseUiModel } from "../recipe-editor/model";
import {
  applyPatches,
  deleteDeclaration,
  duplicateDeclaration,
  insertDeclaration,
  insertStatementAfter,
  renameSymbol,
  setDeclarationKeyword,
  setDoesVerb,
  setMeasuredBy,
  setStatement,
  setStatementList,
  swapDeclarations,
} from "./edits";
import { emitBindings, emitResource } from "./emit";
import type { Outline, OutlineNode } from "./outline";
import {
  findStatement,
  parseOutline,
  recipeNode,
  statementsWithKey,
  stringValue,
  walk,
} from "./outline";
import { loadParser, seed, seeds } from "./test-support";

beforeAll(loadParser);

function recipe(source: string): { outline: Outline; node: OutlineNode } {
  const outline = parseOutline(source);
  const node = recipeNode(outline);
  if (!node) throw new Error("no recipe declaration");
  return { outline, node };
}

function find(parent: OutlineNode, symbol: string): OutlineNode {
  const node = parent.children.find((child) => child.symbol === symbol);
  if (!node) throw new Error(`no declaration for ${symbol}`);
  return node;
}

/** Find a nested operation declaration anywhere in the outline. */
function findOp(outline: Outline, symbol: string): OutlineNode {
  for (const node of walk(outline.nodes)) {
    if (node.keyword === "operation" && node.symbol === symbol) return node;
  }
  throw new Error(`no operation ${symbol}`);
}

/** Find any declaration by symbol anywhere in the outline. */
function findAny(outline: Outline, symbol: string): OutlineNode {
  for (const node of walk(outline.nodes)) {
    if (node.symbol === symbol) return node;
  }
  throw new Error(`no declaration ${symbol}`);
}

describe("golden round-trip", () => {
  /**
   * The tripwire for emitter drift. Re-emit every statement of every seed with
   * the value it already has; anything but byte-identical output means the
   * printer and the grammar disagree.
   */
  it("re-emits every statement of every seed unchanged", () => {
    const all = seeds();
    expect(all).toHaveLength(43);
    let statements = 0;

    for (const { name, source } of all) {
      const outline = parseOutline(source);
      expect(outline.parsed, name).toBe(true);

      for (const parent of walk(outline.nodes)) {
        if (parent.form !== "declaration") continue;
        const counts = new Map<string, number>();
        for (const child of parent.children) {
          counts.set(child.keyword, (counts.get(child.keyword) ?? 0) + 1);
        }
        for (const [keyword, count] of counts) {
          const nodes = statementsWithKey(parent, keyword);
          if (nodes.some((node) => node.form !== "statement")) continue;
          const texts = nodes.map((node) => source.slice(node.codeRange.start, node.codeRange.end));

          if (count === 1) {
            const value = nodes[0].valueRange
              ? source.slice(nodes[0].valueRange.start, nodes[0].valueRange.end)
              : "";
            expect(setStatement(source, parent, keyword, value), `${name}: ${keyword}`).toBe(
              source,
            );
          } else {
            expect(setStatementList(source, parent, keyword, texts), `${name}: ${keyword}`).toBe(
              source,
            );
          }
          statements += count;
        }
      }
    }
    expect(statements).toBeGreaterThan(1000);
  });
});

describe("setStatement", () => {
  const source = seed("baked_macaroni_and_cheese.cg");

  it("replaces a value without disturbing anything else", () => {
    const { node } = recipe(source);
    const macaroni = find(node, "macaroni");
    const next = setStatement(source, macaroni, "quantity", "12 oz");

    expect(next).toContain("quantity 12 oz;");
    expect(next.split("\n")).toHaveLength(source.split("\n").length);
    // The properties and comments the UI model cannot represent are untouched.
    expect(next).toContain("allergen milk;");
    expect(next).toContain("// Butter is also divided");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("appends a new statement before the closing brace", () => {
    const { node } = recipe(source);
    const macaroni = find(node, "macaroni");
    const next = setStatement(source, macaroni, "allergen", "wheat");

    expect(next).toContain("allergen wheat;");
    // Indentation follows the block's existing members.
    expect(next).toMatch(/\n {8}allergen wheat;/);
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("removes a statement when the value is empty", () => {
    const { node } = recipe(source);
    const butter = find(node, "butter");
    const next = setStatement(source, butter, "allergen", "");

    expect(next).not.toContain('allergen milk;\n        name "unsalted butter"');
    expect(next.match(/allergen milk;/g)?.length).toBe(
      (source.match(/allergen milk;/g)?.length ?? 0) - 1,
    );
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("handles an inline block", () => {
    const inline =
      'culinator 0.3;\nrecipe d {\n    title "D";\n    ingredient oil measured by volume { quantity 1 tbsp; }\n}\n';
    const { node } = recipe(inline);
    const oil = find(node, "oil");
    const next = setStatement(inline, oil, "state", "warmed");

    expect(next).toContain("{ quantity 1 tbsp; state warmed; }");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("setStatementList", () => {
  const source = seed("baked_macaroni_and_cheese.cg");

  it("rewrites a whole binding role as a unit", () => {
    const { node } = recipe(source);
    const boil = [...walk(node.children)].find((child) => child.symbol === "boil");
    if (!boil) throw new Error("boil");

    const next = setStatementList(source, boil, "input", [
      "input [macaroni, water];",
      "input salt 2 tbsp;",
    ]);
    expect(next).toContain("input [macaroni, water];");
    expect(next).toContain("input salt 2 tbsp;");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("insertDeclaration", () => {
  const source = seed("pizza_dough.cg");

  it("places a new ingredient after the last one", () => {
    const { node } = recipe(source);
    const declaration = emitResource({
      kind: "ingredient",
      symbol: "semolina",
      measurement: "mass",
      name: "semolina flour",
      quantity: "50 g",
    });
    const next = insertDeclaration(source, node, declaration, "ingredient");

    expect(next).toContain("ingredient semolina measured by mass {");
    expect(next).toContain('name "semolina flour";');
    const model = parseUiModel(next);
    expect(model.diagnostics).toHaveLength(0);
    expect(model.resources.map((resource) => resource.symbol)).toContain("semolina");
    // Inside the recipe block, not appended past its closing brace.
    expect(next.indexOf("semolina")).toBeLessThan(next.lastIndexOf("}"));
  });

  it("keeps a new declaration inside the recipe even with no sibling of its kind", () => {
    const minimal = 'culinator 0.3;\n\nrecipe new_recipe {\n    title "Untitled Recipe";\n}\n';
    const { node } = recipe(minimal);
    const next = insertDeclaration(
      minimal,
      node,
      emitResource({ kind: "ingredient", symbol: "flour", measurement: "mass", quantity: "500 g" }),
      "ingredient",
    );
    const model = parseUiModel(next);
    expect(model.diagnostics).toHaveLength(0);
    expect(model.resources.map((resource) => resource.symbol)).toEqual(["flour"]);
  });
});

describe("deleteDeclaration", () => {
  it("takes the declaration's leading comment with it", () => {
    const source = seed("baked_macaroni_and_cheese.cg");
    const { node } = recipe(source);
    const next = deleteDeclaration(source, find(node, "salt"));

    expect(next).not.toContain("ingredient salt");
    expect(next).not.toContain('// "1 tablespoon plus 1/2 teaspoon kosher salt, divided"');
    // The neighbouring declaration and its own comment survive.
    expect(next).toContain("// Butter is also divided");
    expect(next).toContain("ingredient butter");
  });
});

describe("renameSymbol", () => {
  it("rewrites the declaration and every binding but not prose", () => {
    const source = seed("baked_macaroni_and_cheese.cg");
    const { outline } = recipe(source);
    const next = renameSymbol(source, outline, "salt", "kosher_salt");

    expect(next).toContain("ingredient kosher_salt measured by volume");
    expect(next).toContain("input kosher_salt 1 tbsp;");
    expect(next).not.toMatch(/input salt\b/);
    // A note that merely mentions the word is prose, not a reference.
    for (const line of next.split("\n")) {
      if (line.trim().startsWith("note ")) expect(line).not.toContain("kosher_salt");
    }
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("rewrites references inside a list binding", () => {
    const source = seed("pizza_dough.cg");
    const { outline } = recipe(source);
    const next = renameSymbol(source, outline, "yeast", "dry_yeast");

    expect(next).toContain("ingredient dry_yeast measured by mass");
    expect(next).toContain("input [water, dry_yeast, olive_oil, salt, sugar];");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("duplicateDeclaration", () => {
  it("copies a resource verbatim under a new symbol", () => {
    const source = seed("baked_macaroni_and_cheese.cg");
    const { node } = recipe(source);
    const next = duplicateDeclaration(source, find(node, "butter"), "butter_copy");

    const model = parseUiModel(next);
    expect(model.diagnostics).toHaveLength(0);
    const symbols = model.resources.map((r) => r.symbol);
    expect(symbols).toContain("butter");
    expect(symbols).toContain("butter_copy");
    // The copy carries the unmodelled property too.
    expect(next.match(/allergen milk;/g)?.length).toBe(
      (source.match(/allergen milk;/g)?.length ?? 0) + 1,
    );
  });

  it("only renames the header symbol, not a like-named word in a note", () => {
    const source =
      'culinator 0.3;\nrecipe d {\n    title "D";\n    ingredient salt measured by mass {\n        note "add salt to taste";\n    }\n}\n';
    const { node } = recipe(source);
    const next = duplicateDeclaration(source, find(node, "salt"), "salt_copy");
    // The note text is untouched in both copies.
    expect(next.match(/add salt to taste/g)?.length).toBe(2);
    expect(next).toContain("ingredient salt_copy measured by mass");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("yields", () => {
  const source =
    'culinator 0.3;\nrecipe d {\n    title "D";\n    yield bases measured by count {\n        amount 2 count;\n    }\n}\n';

  it("edits the amount in place", () => {
    const outline = parseOutline(source);
    const next = setStatement(source, findAny(outline, "bases"), "amount", "3 count");
    expect(next).toContain("amount 3 count;");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("changes the measured-by dimension", () => {
    const outline = parseOutline(source);
    const next = setMeasuredBy(source, findAny(outline, "bases"), "mass");
    expect(next).toContain("yield bases measured by mass {");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("formulas", () => {
  const source =
    'culinator 0.3;\nrecipe d {\n    title "D";\n    formula dough as BakersFormula {\n        target 1800 g;\n        ingredient bread_flour as Flour<BakersPercent> {\n            stage final;\n            baker 100%;\n        }\n    }\n}\n';

  it("parses cleanly to begin with", () => {
    expect(parseUiModel(source).diagnostics).toHaveLength(0);
  });

  it("edits an ingredient's baker percentage", () => {
    const outline = parseOutline(source);
    const flour = findAny(outline, "bread_flour");
    const next = setStatement(source, flour, "baker", "80%");
    expect(next).toContain("baker 80%;");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("adds an ingredient into the formula block", () => {
    const outline = parseOutline(source);
    const formula = findAny(outline, "dough");
    const decl =
      "        ingredient water as Liquid<BakersPercent> {\n            stage final;\n            baker 70%;\n        }";
    const next = insertDeclaration(source, formula, decl, "ingredient");
    expect(next).toContain("ingredient water as Liquid<BakersPercent>");
    // The new ingredient is inside the formula block, before the recipe's close.
    expect(next.indexOf("ingredient water")).toBeLessThan(next.indexOf("}\n}"));
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("applyPatches", () => {
  it("applies end-first so earlier offsets stay valid", () => {
    const source = "abcdefghij";
    const result = applyPatches(source, [
      { start: 0, end: 1, replacement: "AAAA" },
      { start: 5, end: 6, replacement: "FF" },
    ]);
    expect(result).toBe("AAAAbcdeFFghij");
  });
});

describe("insertStatementAfter", () => {
  it("places a new metadata line after the title, not at the bottom", () => {
    const source = seed("pizza_dough.cg");
    const { node } = recipe(source);
    const title = findStatement(node, "title");
    if (!title) throw new Error("title");
    const next = insertStatementAfter(source, title, 'byline "Adapted from BeFe";');

    // Immediately after the title line, matching its indentation.
    expect(next).toMatch(/title "Pizza Dough";\n {4}byline "Adapted from BeFe";/);
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("stringValue", () => {
  it("strips the quotes from a string statement but leaves a bare quantity", () => {
    const source = seed("pizza_dough.cg");
    const { node } = recipe(source);
    expect(stringValue(source, findStatement(node, "title")!)).toBe("Pizza Dough");
    expect(stringValue(source, findStatement(node, "active_time")!)).toBe("10 min");
  });
});

describe("setDeclarationKeyword", () => {
  it("changes only the leading keyword, keeping the rest of the header", () => {
    const source = seed("pizza_dough.cg");
    const { node } = recipe(source);
    const next = setDeclarationKeyword(source, find(node, "flour"), "material");

    expect(next).toContain("material flour measured by mass {");
    expect(next).not.toContain("ingredient flour");
    const model = parseUiModel(next);
    expect(model.diagnostics).toHaveLength(0);
    expect(model.resources.find((r) => r.symbol === "flour")?.kind).toBe("material");
  });
});

describe("setMeasuredBy", () => {
  it("adds a measured-by clause where there was none", () => {
    const source = 'culinator 0.3;\nrecipe d {\n    title "D";\n    material dough { }\n}\n';
    const { node } = recipe(source);
    const next = setMeasuredBy(source, find(node, "dough"), "mass");
    expect(next).toContain("material dough measured by mass {");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("replaces an existing dimension and preserves an as-type", () => {
    const source =
      'culinator 0.3;\nrecipe d {\n    title "D";\n    ingredient flour as Flour measured by mass { quantity 1 g; }\n}\n';
    const { node } = recipe(source);
    const next = setMeasuredBy(source, find(node, "flour"), "volume");
    expect(next).toContain("ingredient flour as Flour measured by volume {");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("drops the clause on an empty dimension", () => {
    const source = seed("pizza_dough.cg");
    const { node } = recipe(source);
    const next = setMeasuredBy(source, find(node, "flour"), "");
    expect(next).toContain("ingredient flour {");
    expect(next).not.toContain("ingredient flour measured by");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("setDoesVerb", () => {
  it("replaces an existing verb", () => {
    const source = seed("pizza_dough.cg");
    const outline = parseOutline(source);
    const next = setDoesVerb(source, findOp(outline, "knead"), "combine");
    expect(next).toContain("operation knead does combine {");
    expect(next).not.toContain("operation knead does mix");
    const model = parseUiModel(next);
    expect(model.diagnostics).toHaveLength(0);
    expect(model.operations.find((o) => o.symbol === "knead")?.action).toBe("combine");
  });

  it("adds a verb where there is none", () => {
    const source =
      'culinator 0.3;\nrecipe d {\n    title "D";\n    process p { operation stir { duration 1 min; } }\n}\n';
    const outline = parseOutline(source);
    const next = setDoesVerb(source, findOp(outline, "stir"), "mix");
    expect(next).toContain("operation stir does mix {");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});

describe("operation input bindings", () => {
  // The marquee behaviour: unquantified inputs share one list statement, and
  // each quantified one gets its own — regenerated as a unit.
  it("splits list and single forms and parses back to the same bindings", () => {
    const source = seed("pizza_dough.cg");
    const outline = parseOutline(source);
    const knead = findOp(outline, "knead");
    const lines = emitBindings("input", [
      { symbol: "water" },
      { symbol: "yeast" },
      { symbol: "flour", quantity: "500 g" },
    ]);
    expect(lines).toEqual(["input [water, yeast];", "input flour 500 g;"]);

    const next = setStatementList(source, knead, "input", lines);
    const model = parseUiModel(next);
    expect(model.diagnostics).toHaveLength(0);
    const bindings = model.operations.find((o) => o.symbol === "knead")?.inputBindings;
    expect(bindings).toEqual([
      { symbol: "water" },
      { symbol: "yeast" },
      { symbol: "flour", quantity: "500 g" },
    ]);
  });
});

describe("swapDeclarations", () => {
  it("reorders two resources and still parses", () => {
    const source = seed("pizza_dough.cg");
    const { node } = recipe(source);
    const next = swapDeclarations(source, find(node, "flour"), find(node, "water"));

    expect(next.indexOf("ingredient water")).toBeLessThan(next.indexOf("ingredient flour"));
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
    // Both are still present with their contents intact.
    expect(next).toContain('name "wheat flour"');
    expect(next).toContain('name "lukewarm water"');
  });

  it("carries each declaration's own leading comment along", () => {
    const source = seed("baked_macaroni_and_cheese.cg");
    const { node } = recipe(source);
    const next = swapDeclarations(source, find(node, "salt"), find(node, "butter"));
    // The salt comment stays attached to salt wherever it moved.
    const saltAt = next.indexOf("ingredient salt");
    const saltComment = next.indexOf('// "1 tablespoon plus 1/2 teaspoon kosher salt, divided"');
    expect(saltComment).toBeGreaterThanOrEqual(0);
    expect(saltComment).toBeLessThan(saltAt);
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });
});
