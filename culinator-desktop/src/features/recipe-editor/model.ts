import { parseUiModelWasm } from "../../services/wasm/parser";

export interface SourceRange {
  start: number;
  end: number;
}
/** A syntax problem the parser recovered from, with the byte range to underline. */
export interface UiDiagnostic {
  message: string;
  start: number;
  end: number;
}
export interface UiResource {
  symbol: string;
  name: string;
  kind: string;
  measurement: string;
  quantity?: string;
  /** Qualitative state annotation, e.g. "ripe", "mushy", "chilled". */
  state?: string;
  /** Declared food allergen, e.g. "milk", "egg", or "tree_nut". */
  allergen?: string;
  /** Optional ingredient (e.g. an optional garnish, or "plus more for serving"). */
  optional?: boolean;
  /** One ingredient split across multiple steps ("divided"). */
  divided?: boolean;
  /** Acceptable substitutions, verbatim from the DSL. */
  substitutes?: string[];
  /** "To taste" / "plus more to taste": a base quantity may still be given. */
  toTaste?: boolean;
  /** Size grade for count-measured ingredients ("small", "medium", "large"). */
  size?: string;
  /** Variant-group label; ingredients sharing a label are mutually exclusive. */
  variant?: string;
  /** Free-text handling notes ("seeded and diced", "measured after chopping"). */
  notes?: string[];
  range?: SourceRange;
}
export interface UiProcess {
  symbol: string;
}
/** A structured "cook until…" doneness cue. */
export interface UiDonenessCue {
  kind: string;
  value: string;
}
/** One `input` binding on an operation, optionally with a per-step quantity. */
export interface UiInputBinding {
  symbol: string;
  quantity?: string;
}
export interface UiOperation {
  symbol: string;
  action: string;
  process: string;
  /** Lower bound (or the single fixed value) of the step's duration, in minutes. */
  durationMinutes: number;
  /** Upper bound of the duration when a range was authored, in minutes. */
  durationMaxMinutes?: number;
  labor: string;
  after: string[];
  inputs: string[];
  /** Full input bindings, including per-step amounts for divided ingredients. */
  inputBindings: UiInputBinding[];
  /** Symbols this step binds as `tool`/`container`/`equipment`/`target`. */
  equipment: string[];
  produces?: string;
  /** Numeric temperature setpoint, verbatim (e.g. "350 f"). */
  targetTemperature?: string;
  /** Stovetop heat level (e.g. "medium_high"). */
  heatLevel?: string;
  /** Structured doneness cues. */
  doneness?: UiDonenessCue[];
  /** Per-step image: an asset handle or an external URL, from `photo "…";`. */
  photo?: string;
  /** Batching count: the duration is per-repetition, total is `duration × repeat`. */
  repeat?: number;
  /** Free-text technique notes ("do not rinse", "press wrap on the surface"). */
  notes?: string[];
  range?: SourceRange;
}
export interface UiRecipeModel {
  title: string;
  symbol: string;
  resources: UiResource[];
  processes: UiProcess[];
  operations: UiOperation[];
  source?: string;
  sourceUrl?: string;
  attribution?: string;
  /** The book section (chapter) this recipe belongs to, from `section "…";`. */
  section?: string;
  /** Cover image: an asset handle or an external URL, from `image "…";`. */
  coverImage?: string;
  /** Problems the parser recovered from. Empty for well-formed source. */
  diagnostics: UiDiagnostic[];
}

/**
 * Parse `.cg` source into the editor's UI model.
 *
 * This delegates to `culinator-parser` compiled to WebAssembly, so the DSL has
 * exactly one grammar, one desugaring, and one set of semantics. There used to
 * be a second regex parser here that had to be updated in lockstep with the
 * Rust one; it drifted (a bare `input macaroni;` silently matched nothing, so
 * the ingredient vanished from the step), which is why it is gone.
 *
 * The WASM parser recovers from syntax errors rather than throwing, so a
 * half-typed declaration costs that declaration and nothing else — the live
 * preview keeps rendering. Anything it had to skip is reported in
 * `diagnostics`. `initParser()` must have resolved first; `main.ts` awaits it
 * before mounting.
 */
export function parseUiModel(source: string): UiRecipeModel {
  return parseUiModelWasm(source) as UiRecipeModel;
}
