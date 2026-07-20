import { parseOutlineWasm } from "../../services/wasm/parser";

/**
 * A syntactic map of where every declaration and statement sits in the source.
 *
 * This is the structural counterpart to `recipe-editor/model.ts`. That model
 * says what a recipe *means*, and it is deliberately lossy — it drops yields,
 * servings, formulas, and every property it does not know about, including the
 * `allergen milk;` the seeds really carry. Rebuilding a declaration from it
 * would delete those. The outline says where each piece physically *is*, so an
 * edit to one field splices that field's bytes and everything else is not
 * preserved so much as never touched.
 *
 * Projected from `culinator-parser`'s `Outline` via WebAssembly, so the byte
 * ranges come from the same grammar that reads the file. `culinator-wasm`'s
 * test suite pins these ranges against the UI model's for all 43 seeds, because
 * the builder joins the two by position — if they drift, edits land on the
 * wrong declaration.
 */

export interface OutlineRange {
  start: number;
  end: number;
}

export type OutlineForm = "declaration" | "statement";

export interface OutlineNode {
  /** Leading identifier: `ingredient`, `operation`, `title`, `allergen`. */
  keyword: string;
  form: OutlineForm;
  /** Second token when it is an identifier — the declared or referenced symbol. */
  symbol?: string;
  /** Includes leading trivia (comments, indentation). The span to **delete**. */
  range: OutlineRange;
  /** Excludes leading trivia. The span to **replace**. */
  codeRange: OutlineRange;
  /** Everything after the keyword up to the `;`, for displaying raw values. */
  valueRange?: OutlineRange;
  /** Declarations only: the keyword through to just before the `{`. */
  headerRange?: OutlineRange;
  /** Declarations only: between the braces. New members are appended at its end. */
  blockInnerRange?: OutlineRange;
  /** Whitespace before the node on its line, so insertions match the file. */
  indent: string;
  children: OutlineNode[];
}

export interface Outline {
  /** Document level: the `culinator 0.3;` header and the `recipe` declaration. */
  nodes: OutlineNode[];
  sourceLen: number;
  /**
   * False when the source had no walkable tree — an unbalanced brace or an
   * unterminated string. Callers must keep their last good outline and disable
   * structural editing; an empty `nodes` here does not mean "no declarations".
   */
  parsed: boolean;
}

export function parseOutline(source: string): Outline {
  return parseOutlineWasm(source) as Outline;
}

/** The `recipe` declaration, which owns everything the builder edits. */
export function recipeNode(outline: Outline): OutlineNode | undefined {
  return outline.nodes.find((node) => node.keyword === "recipe");
}

/** Depth-first walk over a node and everything nested inside it. */
export function* walk(nodes: OutlineNode[]): Generator<OutlineNode> {
  for (const node of nodes) {
    yield node;
    yield* walk(node.children);
  }
}

/** First direct child with this keyword. */
export function findStatement(parent: OutlineNode, keyword: string): OutlineNode | undefined {
  return parent.children.find((node) => node.keyword === keyword);
}

/** Every direct child with this keyword — `note` and `input` repeat. */
export function statementsWithKey(parent: OutlineNode, keyword: string): OutlineNode[] {
  return parent.children.filter((node) => node.keyword === keyword);
}

/**
 * The outline node whose code span matches a `UiResource`/`UiOperation` range.
 * This is the join between the two models; both come from the same parse, so a
 * miss means the source changed underneath one of them.
 */
export function findByRange(outline: Outline, range: OutlineRange): OutlineNode | undefined {
  for (const node of walk(outline.nodes)) {
    if (node.codeRange.start === range.start && node.codeRange.end === range.end) return node;
  }
  return undefined;
}

/** The raw text of a node's value — everything after the keyword — or "". */
export function rawValue(source: string, node: OutlineNode): string {
  return node.valueRange ? source.slice(node.valueRange.start, node.valueRange.end) : "";
}

/** A string statement's value with its surrounding quotes removed. */
export function stringValue(source: string, node: OutlineNode): string {
  const raw = rawValue(source, node).trim();
  return raw.startsWith('"') && raw.endsWith('"') && raw.length >= 2 ? raw.slice(1, -1) : raw;
}

/** Every symbol declared or referenced anywhere, for uniquing a new one. */
export function declaredSymbols(outline: Outline): Set<string> {
  const symbols = new Set<string>();
  for (const node of walk(outline.nodes)) {
    if (node.symbol) symbols.add(node.symbol);
  }
  return symbols;
}
