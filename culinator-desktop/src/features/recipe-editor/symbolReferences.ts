import type { SourceRange, UiRecipeModel } from "./model";

export type SymbolKind = "resource" | "operation" | "process" | "yield";

export interface SymbolSpan {
  start: number;
  end: number;
}

export interface SymbolBinding {
  symbol: string;
  kind: SymbolKind;
  definition: SymbolSpan | null;
  uses: SymbolSpan[];
}

export type SymbolReferenceIndex = Map<string, SymbolBinding>;

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** Every whole-word occurrence of `symbol` inside `[from, to)`. */
export function findSymbolSpans(
  source: string,
  symbol: string,
  from = 0,
  to = source.length,
): SymbolSpan[] {
  if (!symbol || from >= to) return [];
  const pattern = new RegExp(`\\b${escapeRegExp(symbol)}\\b`, "g");
  pattern.lastIndex = 0;
  const slice = source.slice(from, to);
  const spans: SymbolSpan[] = [];
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(slice))) {
    spans.push({
      start: from + match.index,
      end: from + match.index + match[0].length,
    });
  }
  return spans;
}

function firstSpanInRange(
  source: string,
  symbol: string,
  range: SourceRange | undefined,
): SymbolSpan | null {
  if (!range) return null;
  return findSymbolSpans(source, symbol, range.start, range.end)[0] ?? null;
}

function pushUnique(spans: SymbolSpan[], next: SymbolSpan[]): void {
  for (const span of next) {
    if (spans.some((existing) => existing.start === span.start && existing.end === span.end)) {
      continue;
    }
    spans.push(span);
  }
}

function ensureBinding(
  index: SymbolReferenceIndex,
  symbol: string,
  kind: SymbolKind,
): SymbolBinding {
  const existing = index.get(symbol);
  if (existing) return existing;
  const created: SymbolBinding = { symbol, kind, definition: null, uses: [] };
  index.set(symbol, created);
  return created;
}

/**
 * Build a DrRacket-style binding index: each declared symbol plus the places
 * operations reference it (`input`, `produces`, `after`, equipment, substitutes).
 *
 * Spans are UTF-16 offsets matching `UiRecipeModel` ranges / JS string indices.
 */
export function buildSymbolReferenceIndex(
  source: string,
  model: UiRecipeModel,
): SymbolReferenceIndex {
  const index: SymbolReferenceIndex = new Map();

  for (const resource of model.resources) {
    const binding = ensureBinding(index, resource.symbol, "resource");
    binding.definition = firstSpanInRange(source, resource.symbol, resource.range);
  }

  for (const operation of model.operations) {
    const binding = ensureBinding(index, operation.symbol, "operation");
    binding.definition = firstSpanInRange(source, operation.symbol, operation.range);
  }

  for (const process of model.processes) {
    // Processes rarely carry a span in the UI model; still register the name so
    // a cursor on `process preparation` can resolve once we find a use/def.
    ensureBinding(index, process.symbol, "process");
  }

  for (const item of model.yields ?? []) {
    ensureBinding(index, item.symbol, "yield");
  }

  for (const operation of model.operations) {
    if (!operation.range) continue;
    const { start, end } = operation.range;
    const referenced = new Set<string>([
      ...operation.inputs,
      ...operation.after,
      ...operation.equipment,
    ]);
    if (operation.produces) referenced.add(operation.produces);

    for (const symbol of referenced) {
      const binding = index.get(symbol);
      if (!binding) continue;
      const spans = findSymbolSpans(source, symbol, start, end).filter((span) => {
        // The operation's own header symbol is its definition, not a use.
        if (
          binding.kind === "operation" &&
          binding.definition &&
          span.start === binding.definition.start
        ) {
          return false;
        }
        return true;
      });
      pushUnique(binding.uses, spans);
    }
  }

  for (const resource of model.resources) {
    if (!resource.range || !resource.substitutes?.length) continue;
    for (const substitute of resource.substitutes) {
      const binding = index.get(substitute);
      if (!binding) continue;
      pushUnique(
        binding.uses,
        findSymbolSpans(source, substitute, resource.range.start, resource.range.end),
      );
    }
  }

  for (const binding of index.values()) {
    binding.uses.sort((left, right) => left.start - right.start);
  }

  return index;
}

/** The binding whose definition or use covers `pos`, if any. */
export function bindingAt(index: SymbolReferenceIndex, pos: number): SymbolBinding | null {
  let best: SymbolBinding | null = null;
  let bestWidth = Number.POSITIVE_INFINITY;
  for (const binding of index.values()) {
    const spans = binding.definition ? [binding.definition, ...binding.uses] : binding.uses;
    for (const span of spans) {
      if (pos < span.start || pos > span.end) continue;
      const width = span.end - span.start;
      if (width < bestWidth) {
        best = binding;
        bestWidth = width;
      }
    }
  }
  return best;
}

/** Identifier under `pos` (CodeMirror allows the cursor to sit on either edge). */
export function identifierAt(source: string, pos: number): string | null {
  if (!source) return null;
  const isIdent = (ch: string) => /[A-Za-z0-9_]/.test(ch);
  let start = pos;
  let end = pos;
  if (start > 0 && !isIdent(source[start] ?? "") && isIdent(source[start - 1] ?? "")) {
    start -= 1;
    end = start + 1;
  }
  if (!isIdent(source[start] ?? "")) return null;
  while (start > 0 && isIdent(source[start - 1] ?? "")) start -= 1;
  while (end < source.length && isIdent(source[end] ?? "")) end += 1;
  const word = source.slice(start, end);
  return word || null;
}

export function bindingForCursor(
  index: SymbolReferenceIndex,
  source: string,
  pos: number,
): SymbolBinding | null {
  const direct = bindingAt(index, pos);
  if (direct) return direct;
  const word = identifierAt(source, pos);
  return word ? (index.get(word) ?? null) : null;
}

/** True when `pos` sits inside a `//` comment or a double-quoted string. */
export function isInsideStringOrComment(source: string, pos: number): boolean {
  let i = 0;
  while (i < source.length && i <= pos) {
    if (source.startsWith("//", i)) {
      const end = source.indexOf("\n", i);
      const lineEnd = end === -1 ? source.length : end;
      if (pos >= i && pos < lineEnd) return true;
      i = lineEnd;
      continue;
    }
    if (source[i] === '"') {
      let j = i + 1;
      while (j < source.length && source[j] !== '"' && source[j] !== "\n") {
        if (source[j] === "\\") j += 1;
        j += 1;
      }
      const end = Math.min(j + 1, source.length);
      if (pos >= i && pos < end) return true;
      i = end;
      continue;
    }
    i += 1;
  }
  return false;
}

export interface DefinedSymbolSpan extends SymbolSpan {
  kind: SymbolKind;
}

/**
 * Every occurrence of a recipe-defined name in source, for ambient coloring.
 * Skips matches inside comments and strings so prose in notes stays plain.
 */
export function definedSymbolSpans(
  source: string,
  index: SymbolReferenceIndex,
): DefinedSymbolSpan[] {
  const spans: DefinedSymbolSpan[] = [];
  for (const binding of index.values()) {
    for (const span of findSymbolSpans(source, binding.symbol)) {
      if (isInsideStringOrComment(source, span.start)) continue;
      spans.push({ ...span, kind: binding.kind });
    }
  }
  spans.sort((left, right) => left.start - right.start || left.end - right.end);
  return spans;
}
