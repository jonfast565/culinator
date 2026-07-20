/**
 * Printing `.cg` source. This is the only module in the frontend that knows DSL
 * syntax, and it is deliberately free of Vue and WASM imports so it can be
 * tested as plain functions.
 *
 * Grammar reference: `docs/GRAMMAR.ebnf`; the reader is
 * `culinator-parser/src/semantic.rs`. There is no emitter on the Rust side to
 * mirror — printing is a capability the workspace did not previously have — so
 * this file is exposed to exactly the drift that killed the old TypeScript
 * parser (see `recipe-editor/model.ts`). The tripwire is the golden round-trip
 * test in `emit.test.ts`: it re-emits every declaration of all 43 seed recipes
 * and requires byte-identical output, so a grammar change that alters seed
 * syntax fails immediately.
 *
 * Whole declarations are printed only when creating something new. Editing an
 * existing one splices a single statement — see `edits.ts`.
 */

/**
 * Make a value safe to sit inside a `"…"` literal.
 *
 * The DSL has no string escaping. Both lexers *consume* a `\"` so it does not
 * terminate the token, but neither unescapes it — `semantic.rs` stores the raw
 * slice — so a backslash written here survives into the title, the narrative,
 * and the EPUB. (`culinator-import`'s `quote()` emits `\"` and is wrong for
 * exactly this reason; do not port it.) So: substitute rather than escape.
 */
export function sanitizeString(value: string): string {
  return (
    value
      .replaceAll('"', "'")
      .replace(/\s+/g, " ")
      // A trailing backslash would swallow the closing quote.
      .replace(/\\+$/, "")
      .trim()
  );
}

/** A string value ready to embed: `"kosher salt"`. */
export function quote(value: string): string {
  return `"${sanitizeString(value)}"`;
}

/**
 * Keywords a symbol must not collide with. A declaration keyword in symbol
 * position reparses as a different construct entirely — `symbolize("Yield")`
 * would turn an ingredient into a `yield_decl`.
 */
const RESERVED = new Set([
  "type",
  "resource",
  "ingredient",
  "material",
  "container",
  "equipment",
  "environment",
  "labor",
  "process",
  "operation",
  "prep",
  "serving",
  "yield",
  "formula",
  "recipe",
  "book",
  "recipe_book",
  "as",
  "measured",
  "by",
  "into",
  "does",
  "after",
  "input",
  "output",
  "produces",
  "target",
  "tool",
  "to",
  "relative",
  "of",
  "total",
]);

/**
 * Derive a DSL identifier from a display name.
 *
 * Ported from `culinator-import`'s `symbolize`, which only ever ran on a
 * one-shot import and so needed neither uniquing nor a keyword guard. A builder
 * needs both: `taken` is every symbol already in the document.
 */
export function symbolize(name: string, taken: Set<string> = new Set(), fallback = "item"): string {
  let out = "";
  for (const character of name) {
    if (/[a-zA-Z0-9]/.test(character)) {
      out += character.toLowerCase();
    } else if (out.length > 0 && !out.endsWith("_")) {
      out += "_";
    }
  }
  let base = out.replace(/^_+|_+$/g, "");
  // Non-ASCII names ("🥕", "焼き") symbolize to nothing.
  if (!base) base = fallback;
  if (/^[0-9]/.test(base)) base = `${fallback}_${base}`;
  if (RESERVED.has(base)) base = `${base}_1`;
  if (!taken.has(base)) return base;
  for (let suffix = 2; ; suffix += 1) {
    const candidate = `${base}_${suffix}`;
    if (!taken.has(candidate)) return candidate;
  }
}

/**
 * Print a number the way the parser's projection does: no trailing `.0`, and no
 * float noise (`0.30000000000000004`) leaking into the source.
 */
export function formatNumber(value: number): string {
  if (!Number.isFinite(value)) return "0";
  if (Number.isInteger(value)) return String(value);
  return String(Number(value.toFixed(4)));
}

/**
 * `"400 g"`. Units are not validated: the grammar keeps an unrecognized unit
 * verbatim and classifies it as `ratio`, so a permissive free-text unit field
 * is correct — a dropdown that rejects would be narrower than the language.
 */
export function formatQuantity(value: number | string, unit?: string): string {
  const amount = typeof value === "number" ? formatNumber(value) : value.trim();
  const cleanUnit = unit?.trim();
  return cleanUnit ? `${amount} ${cleanUnit}` : amount;
}

/** `"2 to 3 clove"` — the upper bound carries the unit. */
export function formatRange(min: number | string, max: number | string, unit?: string): string {
  const low = typeof min === "number" ? formatNumber(min) : min.trim();
  return `${low} to ${formatQuantity(max, unit)}`;
}

/** `[a, b]`. */
export function formatList(items: string[]): string {
  return `[${items.join(", ")}]`;
}

/** One statement: `quantity 400 g;`. A blank value yields a bare `optional;`. */
export function emitStatement(keyword: string, value?: string): string {
  const text = value?.trim();
  return text ? `${keyword} ${text};` : `${keyword};`;
}

export interface ResourceDraft {
  kind: string;
  symbol: string;
  /** Dimension for `measured by`; omitted when absent. */
  measurement?: string;
  name?: string;
  quantity?: string;
  state?: string;
  optional?: boolean;
  divided?: boolean;
  toTaste?: boolean;
  size?: string;
  variant?: string;
  substitutes?: string[];
  notes?: string[];
}

/**
 * A whole resource declaration, for a resource that does not exist yet.
 *
 * `divided` deliberately does not print a `quantity`: a divided ingredient's
 * amounts live on the step bindings, and declaring one here as well double
 * counts it.
 */
export function emitResource(draft: ResourceDraft, indent = "    "): string {
  const inner = `${indent}    `;
  const header = draft.measurement
    ? `${draft.kind} ${draft.symbol} measured by ${draft.measurement}`
    : `${draft.kind} ${draft.symbol}`;
  const body: string[] = [];
  if (draft.name) body.push(emitStatement("name", quote(draft.name)));
  if (draft.quantity && !draft.divided) body.push(emitStatement("quantity", draft.quantity));
  if (draft.state) body.push(emitStatement("state", draft.state));
  if (draft.size) body.push(emitStatement("size", quote(draft.size)));
  if (draft.variant) body.push(emitStatement("variant", quote(draft.variant)));
  if (draft.optional) body.push(emitStatement("optional", "true"));
  if (draft.divided) body.push(emitStatement("divided", "true"));
  if (draft.toTaste) body.push(emitStatement("to_taste", "true"));
  if (draft.substitutes?.length) {
    body.push(emitStatement("substitutes", formatList(draft.substitutes)));
  }
  for (const note of draft.notes ?? []) body.push(emitStatement("note", quote(note)));

  if (!body.length) return `${indent}${header} { }`;
  return `${indent}${header} {\n${body.map((line) => `${inner}${line}`).join("\n")}\n${indent}}`;
}

/** One `input`/`output`/`tool`/… binding target, optionally with an amount. */
export interface BindingDraft {
  symbol: string;
  quantity?: string;
}

/**
 * Binding statements for one role.
 *
 * The grammar's list form carries no amount (`input [flour 400 g];` is not
 * legal), so unquantified targets share a single list statement and each
 * quantified one gets its own — which is how the seeds express a divided
 * ingredient:
 *
 * ```text
 * input [macaroni];
 * input salt 1 tbsp;
 * ```
 *
 * Because that constraint spans statements, callers regenerate a role's
 * bindings as a unit rather than patching one of them.
 */
export function emitBindings(role: string, bindings: BindingDraft[]): string[] {
  const bare = bindings.filter((binding) => !binding.quantity?.trim());
  const measured = bindings.filter((binding) => binding.quantity?.trim());
  const lines: string[] = [];
  if (bare.length) {
    lines.push(emitStatement(role, formatList(bare.map((binding) => binding.symbol))));
  }
  for (const binding of measured) {
    lines.push(emitStatement(role, `${binding.symbol} ${binding.quantity?.trim()}`));
  }
  return lines;
}

export interface OperationDraft {
  symbol: string;
  /** The `does <verb>` action. */
  action?: string;
  inputs?: BindingDraft[];
  produces?: string;
  equipment?: BindingDraft[];
  after?: string[];
  duration?: string;
  labor?: string;
  temperature?: string;
  heat?: string;
  repeat?: number;
  optional?: boolean;
  notes?: string[];
}

/**
 * A whole operation declaration.
 *
 * Naming matters for output quality: `culinator-narrative`'s generic actions
 * (`mix`, `heat`, `rest`, `move`, `strain`, `coat`) fall back to humanizing the
 * operation's *symbol*, which is why the seeds write `make_roux does heat`
 * rather than leaning on the verb. Callers should encourage a descriptive
 * symbol.
 */
export function emitOperation(draft: OperationDraft, indent = "        "): string {
  const inner = `${indent}    `;
  const header = draft.action
    ? `operation ${draft.symbol} does ${draft.action}`
    : `operation ${draft.symbol}`;
  const body: string[] = [];
  body.push(...emitBindings("input", draft.inputs ?? []));
  if (draft.after?.length) body.push(emitStatement("after", formatList(draft.after)));
  body.push(...emitBindings("equipment", draft.equipment ?? []));
  if (draft.temperature) body.push(emitStatement("temperature", draft.temperature));
  if (draft.heat) body.push(emitStatement("heat", draft.heat));
  if (draft.duration) body.push(emitStatement("duration", draft.duration));
  if (draft.labor) body.push(emitStatement("labor", draft.labor));
  if (draft.repeat != null) body.push(emitStatement("repeat", formatNumber(draft.repeat)));
  if (draft.optional) body.push(emitStatement("optional", "true"));
  for (const note of draft.notes ?? []) body.push(emitStatement("note", quote(note)));
  if (draft.produces) body.push(emitStatement("produces", draft.produces));

  if (!body.length) return `${indent}${header} { }`;
  return `${indent}${header} {\n${body.map((line) => `${inner}${line}`).join("\n")}\n${indent}}`;
}

/** An empty named process, ready for operations to be added into it. */
export function emitProcess(symbol: string, indent = "    "): string {
  return `${indent}process ${symbol} {\n${indent}}`;
}

export interface YieldDraft {
  /** `yield` or `serving`. */
  keyword?: string;
  symbol: string;
  measurement?: string;
  /** e.g. `"2 count"`. */
  amount?: string;
}

export function emitYield(draft: YieldDraft, indent = "    "): string {
  const keyword = draft.keyword ?? "yield";
  const header = draft.measurement
    ? `${keyword} ${draft.symbol} measured by ${draft.measurement}`
    : `${keyword} ${draft.symbol}`;
  if (!draft.amount) return `${indent}${header} { }`;
  return `${indent}${header} {\n${indent}    ${emitStatement("amount", draft.amount)}\n${indent}}`;
}

export interface FormulaIngredientDraft {
  symbol: string;
  /** Type reference, e.g. `Flour<BakersPercent>`. */
  type?: string;
  stage?: string;
  /** Baker's percentage, e.g. `"80%"`. */
  baker?: string;
}

export interface FormulaDraft {
  symbol: string;
  /** `as <TypeRef>`, `relative to <id>`, or `of total`. */
  basis?: string;
  /** Total dough weight, e.g. `"1800 g"`. */
  target?: string;
  ingredients?: FormulaIngredientDraft[];
}

export function emitFormula(draft: FormulaDraft, indent = "    "): string {
  const inner = `${indent}    `;
  const header = draft.basis ? `formula ${draft.symbol} ${draft.basis}` : `formula ${draft.symbol}`;
  const body: string[] = [];
  if (draft.target) body.push(`${inner}${emitStatement("target", draft.target)}`);
  for (const ingredient of draft.ingredients ?? []) {
    const head = ingredient.type
      ? `ingredient ${ingredient.symbol} as ${ingredient.type}`
      : `ingredient ${ingredient.symbol}`;
    const fields: string[] = [];
    if (ingredient.stage) fields.push(emitStatement("stage", ingredient.stage));
    if (ingredient.baker) fields.push(emitStatement("baker", ingredient.baker));
    body.push(
      fields.length
        ? `${inner}${head} {\n${fields.map((line) => `${inner}    ${line}`).join("\n")}\n${inner}}`
        : `${inner}${head} { }`,
    );
  }
  if (!body.length) return `${indent}${header} { }`;
  return `${indent}${header} {\n${body.join("\n")}\n${indent}}`;
}
