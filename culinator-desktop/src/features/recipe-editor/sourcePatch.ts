// Targeted edits to `.cg` source text for structured "edit in place" fields.
// The DSL source is the source of truth, so rather than regenerate it (which
// would drop comments/formatting/unknown constructs) we patch the specific
// property span, or insert a new property near the top of the recipe block.

import type { SourceRange, UiOperation, UiResource } from "./model";

interface SourcePatch {
  start: number;
  end: number;
  replacement: string;
}

function applyPatches(source: string, patches: SourcePatch[]): string {
  const ordered = [...patches].sort((left, right) => right.start - left.start);
  let result = source;
  for (const patch of ordered) {
    result = `${result.slice(0, patch.start)}${patch.replacement}${result.slice(patch.end)}`;
  }
  return result;
}

function quantityPatchInSpan(
  source: string,
  spanStart: number,
  spanEnd: number,
  newQuantity: string,
  propertyPattern: RegExp,
): SourcePatch | null {
  const span = source.slice(spanStart, spanEnd);
  const match = propertyPattern.exec(span);
  if (!match || match.index == null) return null;
  const start = spanStart + match.index;
  return {
    start,
    end: start + match[0].length,
    replacement: `${match[1]} ${newQuantity};`,
  };
}

function sanitize(value: string): string {
  // The DSL has no string escaping, so keep values quote-safe and single-line.
  return value.replaceAll('"', "'").replace(/\s+/g, " ").trim();
}

/**
 * Set a recipe-level string property (e.g. `title`, `section`). Replaces an
 * existing `key "…";`, inserts one just after `title` (or the recipe brace) if
 * absent, or removes it when `value` is empty.
 */
export function setRecipeProperty(source: string, key: string, value: string): string {
  const clean = sanitize(value);
  const existing = new RegExp(`^([ \\t]*)${key}\\s+"[^"]*"\\s*;`, "m");

  if (!clean) {
    // Remove the property line entirely (and its trailing newline) if present.
    return source.replace(new RegExp(`^[ \\t]*${key}\\s+"[^"]*"\\s*;\\n?`, "m"), "");
  }
  if (existing.test(source)) {
    return source.replace(existing, `$1${key} "${clean}";`);
  }

  // Insert after the `title "…";` line, matching its indentation.
  const titleLine = /^([ \t]*)title\s+"[^"]*"\s*;/m;
  const titleMatch = titleLine.exec(source);
  if (titleMatch) {
    const indent = titleMatch[1];
    const insertAt = titleMatch.index + titleMatch[0].length;
    return `${source.slice(0, insertAt)}\n${indent}${key} "${clean}";${source.slice(insertAt)}`;
  }

  // Fall back to inserting just inside the recipe block.
  const brace = /recipe\s+[A-Za-z0-9_]+\s*\{/m;
  const braceMatch = brace.exec(source);
  if (braceMatch) {
    const insertAt = braceMatch.index + braceMatch[0].length;
    return `${source.slice(0, insertAt)}\n    ${key} "${clean}";${source.slice(insertAt)}`;
  }
  return source;
}

/**
 * Set (or clear, when `value` is empty) a single operation's per-step
 * `photo "…";` property, patching only within that operation's source span so
 * the rest of the recipe is left byte-for-byte untouched. Replaces an existing
 * `photo`, otherwise inserts one before the block's closing brace — or, for a
 * block-less `prep … ;` statement, wraps it in a block that carries the photo.
 */
export function setOperationPhoto(source: string, range: SourceRange, value: string): string {
  const clean = sanitize(value);
  const before = source.slice(0, range.start);
  let span = source.slice(range.start, range.end);
  const after = source.slice(range.end);

  const existing = /([ \t]*)photo\s+"[^"]*"\s*;/;

  if (!clean) {
    // Drop the photo statement (with its own leading newline) if present.
    return before + span.replace(/\n?[ \t]*photo\s+"[^"]*"\s*;/, "") + after;
  }
  if (existing.test(span)) {
    return before + span.replace(existing, `$1photo "${clean}";`) + after;
  }

  const closeBrace = span.lastIndexOf("}");
  if (closeBrace !== -1) {
    const openBrace = span.indexOf("{");
    const multiline = openBrace !== -1 && span.slice(openBrace, closeBrace).includes("\n");
    if (multiline) {
      // Add an indented line just before the closing brace's line, one level
      // deeper than the `operation` header.
      const headerIndent = /^([ \t]*)/.exec(span)?.[1] ?? "";
      const indent = `${headerIndent}    `;
      const lineStart = span.lastIndexOf("\n", closeBrace) + 1;
      span = `${span.slice(0, lineStart)}${indent}photo "${clean}";\n${span.slice(lineStart)}`;
    } else {
      // Inline block (`… { input a; }`): tuck the photo in before the brace.
      const head = span.slice(0, closeBrace).replace(/\s*$/, "");
      const sep = head.endsWith(";") || head.endsWith("{") ? " " : "; ";
      span = `${head}${sep}photo "${clean}"; ${span.slice(closeBrace)}`;
    }
    return before + span + after;
  }

  // Block-less prep statement (`prep foo bar;`): give it a block for the photo.
  const semi = span.lastIndexOf(";");
  if (semi !== -1) {
    span = `${span.slice(0, semi)} { photo "${clean}"; }${span.slice(semi + 1)}`;
    return before + span + after;
  }
  return source;
}

const QUANTITY_PROPERTY = /\b(quantity|mass|amount)\s+([^;]+)\s*;/;
const TEMPERATURE_PROPERTY = /\btemperature\s+([\d.]+)\s+([A-Za-z_]+)\s*;/;

/**
 * Rewrite convertible ingredient quantities and step temperatures in place.
 * Patches are applied from the end of the file forward so byte ranges stay valid.
 */
export async function convertRecipeQuantitiesInSource(
  source: string,
  ingredients: UiResource[],
  operations: UiOperation[],
  convertQuantity: (text: string) => Promise<string | null>,
): Promise<string> {
  const patches: SourcePatch[] = [];

  for (const ingredient of ingredients) {
    if (!ingredient.quantity || !ingredient.range) continue;
    const converted = await convertQuantity(ingredient.quantity);
    if (!converted || converted === ingredient.quantity) continue;
    const patch = quantityPatchInSpan(
      source,
      ingredient.range.start,
      ingredient.range.end,
      converted,
      QUANTITY_PROPERTY,
    );
    if (patch) patches.push(patch);
  }

  for (const operation of operations) {
    if (!operation.targetTemperature || !operation.range) continue;
    const converted = await convertQuantity(operation.targetTemperature);
    if (!converted || converted === operation.targetTemperature) continue;
    const span = source.slice(operation.range.start, operation.range.end);
    const match = TEMPERATURE_PROPERTY.exec(span);
    if (!match || match.index == null) continue;
    const parts = converted.match(/^([\d./]+)\s+([A-Za-z_]+)$/);
    if (!parts) continue;
    const start = operation.range.start + match.index;
    patches.push({
      start,
      end: start + match[0].length,
      replacement: `temperature ${parts[1]} ${parts[2]};`,
    });
  }

  if (!patches.length) return source;
  return applyPatches(source, patches);
}

/** Remove one operation or prep statement from source using its byte range. */
export function deleteOperationFromSource(source: string, operation: UiOperation): string | null {
  if (!operation.range) return null;
  const { start, end } = operation.range;
  return `${source.slice(0, start)}${source.slice(end)}`;
}
