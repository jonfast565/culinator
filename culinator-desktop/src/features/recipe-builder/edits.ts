/**
 * Structural edits to `.cg` source, expressed as byte-range splices over the
 * outline.
 *
 * Every function here is `(source, …) => string`, pure and **synchronous**.
 * That is load-bearing, not stylistic: a range is only valid for the exact
 * source it was computed from, so awaiting between reading a range and splicing
 * it lets an interleaved edit invalidate it. One snapshot, one commit. Multi-
 * field changes therefore collect patches and apply them end-first (see
 * `applyPatches`), which keeps earlier offsets valid — the same discipline
 * `recipe-editor/sourcePatch.ts` uses.
 *
 * The design rule: to change a field, replace that statement's `codeRange`.
 * Leading trivia sits outside `codeRange`, so indentation and the comment above
 * a declaration survive; siblings are never re-emitted, so unmodelled
 * properties like `allergen milk;` and hand-chosen property ordering survive
 * too. A whole block is printed only when creating something new.
 */

import { emitStatement } from "./emit";
import type { Outline, OutlineNode, OutlineRange } from "./outline";
import { recipeNode, statementsWithKey, walk } from "./outline";

export interface SourcePatch {
  start: number;
  end: number;
  replacement: string;
}

/**
 * Apply patches end-first so that earlier offsets stay valid. Patches must not
 * overlap; overlapping ones would corrupt each other silently.
 */
export function applyPatches(source: string, patches: SourcePatch[]): string {
  const ordered = [...patches].sort((left, right) => right.start - left.start);
  let result = source;
  for (const patch of ordered) {
    result = `${result.slice(0, patch.start)}${patch.replacement}${result.slice(patch.end)}`;
  }
  return result;
}

function splice(source: string, range: OutlineRange, replacement: string): string {
  return `${source.slice(0, range.start)}${replacement}${source.slice(range.end)}`;
}

/** Indentation for members of `parent`, following the file's own style. */
function childIndent(parent: OutlineNode): string {
  const existing = parent.children.find((child) => child.indent);
  if (existing) return existing.indent;
  return `${parent.indent}    `;
}

/** Whether a declaration's block is written on one line: `{ quantity 1 tbsp; }`. */
function isInline(source: string, parent: OutlineNode): boolean {
  const inner = parent.blockInnerRange;
  return inner != null && !source.slice(inner.start, inner.end).includes("\n");
}

/**
 * Set, insert, or remove a single-valued statement inside a declaration.
 *
 * An existing statement is replaced in place. A new one is appended just before
 * the closing brace, which is the least surprising position and leaves existing
 * ordering undisturbed. An empty `value` removes the statement along with its
 * leading trivia.
 */
export function setStatement(
  source: string,
  parent: OutlineNode,
  keyword: string,
  value: string | null | undefined,
): string {
  const existing = parent.children.find((child) => child.keyword === keyword);
  const text = value?.trim();

  if (!text) {
    return existing ? splice(source, existing.range, "") : source;
  }
  const statement = emitStatement(keyword, text);
  if (existing) return splice(source, existing.codeRange, statement);

  const inner = parent.blockInnerRange;
  if (!inner) return source;
  if (isInline(source, parent)) {
    const head = source.slice(inner.start, inner.end).replace(/\s*$/, "");
    const separator = head.endsWith(";") || head === "" ? " " : "; ";
    return splice(source, inner, `${head}${separator}${statement} `);
  }
  const indent = childIndent(parent);
  const tail = parent.children.at(-1)?.range.end ?? inner.start;
  return `${source.slice(0, tail)}\n${indent}${statement}${source.slice(tail)}`;
}

/**
 * Replace every statement with this keyword by a new set of lines.
 *
 * Repeatable keys (`note`, and every binding role) cannot be patched one at a
 * time when the change is cross-statement — moving an input from the list form
 * to the amount-carrying single form touches two statements at once — so the
 * whole role is regenerated. The first existing statement's position is reused
 * so the new lines land where the old ones were.
 */
export function setStatementList(
  source: string,
  parent: OutlineNode,
  keyword: string,
  lines: string[],
): string {
  const existing = statementsWithKey(parent, keyword);
  if (!existing.length && !lines.length) return source;

  const indent = existing[0]?.indent || childIndent(parent);
  const inline = isInline(source, parent);
  const joined = inline
    ? lines.join(" ")
    : lines.map((line, index) => (index === 0 ? line : `${indent}${line}`)).join("\n");

  if (existing.length) {
    const patches: SourcePatch[] = existing
      .slice(1)
      .map((node) => ({ start: node.range.start, end: node.range.end, replacement: "" }));
    patches.push(
      lines.length
        ? {
            start: existing[0].codeRange.start,
            end: existing[0].codeRange.end,
            replacement: joined,
          }
        : { start: existing[0].range.start, end: existing[0].range.end, replacement: "" },
    );
    return applyPatches(source, patches);
  }

  const inner = parent.blockInnerRange;
  if (!inner || !lines.length) return source;
  if (inline) {
    const head = source.slice(inner.start, inner.end).replace(/\s*$/, "");
    const separator = head.endsWith(";") || head === "" ? " " : "; ";
    return splice(source, inner, `${head}${separator}${joined} `);
  }
  const tail = parent.children.at(-1)?.range.end ?? inner.start;
  const block = lines.map((line) => `\n${indent}${line}`).join("");
  return `${source.slice(0, tail)}${block}${source.slice(tail)}`;
}

/**
 * Insert a declaration into `parent`'s block, after the last child sharing
 * `afterKeyword` when there is one.
 *
 * `declaration` arrives already indented by the `emit*` helpers. Anything
 * appended past the recipe's closing brace instead is ignored by the parser
 * *without a diagnostic*, so placement has to be explicit.
 */
export function insertDeclaration(
  source: string,
  parent: OutlineNode,
  declaration: string,
  afterKeyword?: string,
): string {
  const inner = parent.blockInnerRange;
  if (!inner) return source;
  const siblings = afterKeyword ? statementsWithKey(parent, afterKeyword) : [];
  const anchor = siblings.at(-1) ?? parent.children.at(-1);
  const at = anchor?.range.end ?? inner.start;
  return `${source.slice(0, at)}\n${declaration}${source.slice(at)}`;
}

/**
 * Insert a statement immediately after `anchor`, matching its indentation.
 *
 * Used to keep recipe-level metadata grouped near the top: a new `section` is
 * placed after `title` rather than appended at the bottom of the block, where
 * `setStatement` would put it, past the ingredients and yields.
 */
export function insertStatementAfter(
  source: string,
  anchor: OutlineNode,
  statement: string,
): string {
  const at = anchor.range.end;
  return `${source.slice(0, at)}\n${anchor.indent}${statement}${source.slice(at)}`;
}

/**
 * Remove a declaration or statement, including its leading trivia — so the
 * comment explaining an ingredient goes with the ingredient rather than being
 * orphaned above the next one.
 */
export function deleteDeclaration(source: string, node: OutlineNode): string {
  return splice(source, node.range, "");
}

/**
 * Change a declaration's leading keyword — the ingredient/material/container
 * distinction — by splicing just that first token. Everything else in the
 * header (`flour measured by mass`) is left exactly as it was.
 */
export function setDeclarationKeyword(source: string, node: OutlineNode, keyword: string): string {
  const start = node.codeRange.start;
  return `${source.slice(0, start)}${keyword}${source.slice(start + node.keyword.length)}`;
}

/**
 * Set, change, or remove the `measured by <dimension>` clause on a declaration
 * header, leaving any `as <Type>` and the rest of the header intact. A blank
 * dimension drops the clause.
 */
export function setMeasuredBy(source: string, node: OutlineNode, dimension: string): string {
  const header = node.headerRange;
  if (!header) return source;
  const text = source.slice(header.start, header.end);
  const trailing = /\s*$/.exec(text)?.[0] ?? "";
  let core = text.slice(0, text.length - trailing.length).replace(/\s+measured\s+by\s+\w+/, "");
  const clean = dimension.trim();
  if (clean) core += ` measured by ${clean}`;
  return splice(source, header, core + (trailing || " "));
}

/**
 * Set, change, or remove the `does <verb>` clause on an operation header.
 *
 * Leaves the rest of the header alone. An `as <Type>` header is left untouched
 * (the grammar is `does` OR `as`, so a verb is not appended alongside a type).
 */
export function setDoesVerb(source: string, node: OutlineNode, verb: string): string {
  const header = node.headerRange;
  if (!header) return source;
  const text = source.slice(header.start, header.end);
  const trailing = /\s*$/.exec(text)?.[0] ?? "";
  let core = text.slice(0, text.length - trailing.length).replace(/\s+does\s+\w+/, "");
  const clean = verb.trim();
  if (clean && !/\s+as\s+/.test(core)) core += ` does ${clean}`;
  return splice(source, header, core + (trailing || " "));
}

/**
 * Swap two declarations, each moving with its own leading comment (which the
 * outline bundles into `range`). Used for nudging a resource up or down; the
 * two need not be adjacent, since the patches are applied independently.
 */
export function swapDeclarations(source: string, first: OutlineNode, second: OutlineNode): string {
  return applyPatches(source, [
    {
      start: first.range.start,
      end: first.range.end,
      replacement: source.slice(second.range.start, second.range.end),
    },
    {
      start: second.range.start,
      end: second.range.end,
      replacement: source.slice(first.range.start, first.range.end),
    },
  ]);
}

/**
 * Insert a copy of a declaration right after it, with a new symbol.
 *
 * The copy is the declaration's exact text — every property, comment, and
 * unmodelled construct — with only the header symbol swapped, so a duplicated
 * ingredient keeps everything the builder does not model. References elsewhere
 * are intentionally not repointed; the copy is a fresh, independent declaration.
 */
export function duplicateDeclaration(source: string, node: OutlineNode, newSymbol: string): string {
  const text = source.slice(node.codeRange.start, node.codeRange.end);
  const symbol = node.symbol;
  // Swap the symbol only where it sits in the header — right after the keyword —
  // so an occurrence of the same word inside a note or value is left alone.
  const renamed = symbol
    ? text.replace(
        new RegExp(`^(\\s*${escapeRegExp(node.keyword)}\\s+)${escapeRegExp(symbol)}\\b`),
        `$1${newSymbol}`,
      )
    : text;
  const at = node.range.end;
  return `${source.slice(0, at)}\n${node.indent}${renamed}${source.slice(at)}`;
}

/**
 * Rename a symbol at its declaration and at every reference.
 *
 * References are found through the outline's `symbol` field, never by scanning
 * the text: several seeds mention an ingredient by name inside a `note "…"`
 * ("season with salt"), and a whole-file find-and-replace would rewrite the
 * prose as well as the binding.
 */
export function renameSymbol(source: string, outline: Outline, from: string, to: string): string {
  if (!from || !to || from === to) return source;
  const patches: SourcePatch[] = [];
  for (const node of walk(outline.nodes)) {
    if (node.symbol !== from) continue;
    // The symbol is the second token, so it starts after the keyword.
    const start = node.valueRange?.start ?? node.codeRange.start + node.keyword.length + 1;
    patches.push({ start, end: start + from.length, replacement: to });
  }
  // A list binding (`input [flour, water];`) holds references the outline does
  // not surface individually; rewrite identifiers inside the brackets only.
  for (const node of walk(outline.nodes)) {
    const value = node.valueRange;
    if (!value) continue;
    const text = source.slice(value.start, value.end);
    if (!text.startsWith("[")) continue;
    const pattern = new RegExp(`(?<![A-Za-z0-9_])${escapeRegExp(from)}(?![A-Za-z0-9_])`, "g");
    for (const match of text.matchAll(pattern)) {
      patches.push({
        start: value.start + (match.index ?? 0),
        end: value.start + (match.index ?? 0) + from.length,
        replacement: to,
      });
    }
  }
  return applyPatches(source, dedupe(patches));
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** Drop patches that target a range already covered, so splices never overlap. */
function dedupe(patches: SourcePatch[]): SourcePatch[] {
  const seen = new Set<string>();
  return patches.filter((patch) => {
    const key = `${patch.start}:${patch.end}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

/**
 * The recipe declaration, or `undefined` when the source has none to edit.
 * Callers use this to gate every mutation: an outline with `parsed: false` has
 * no nodes, and editing must be disabled rather than silently no-op.
 */
export function editableRecipe(outline: Outline): OutlineNode | undefined {
  return outline.parsed ? recipeNode(outline) : undefined;
}
