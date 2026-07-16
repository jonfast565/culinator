export interface SourceRange {
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
  /** Optional ingredient (e.g. an optional garnish, or "plus more for serving"). */
  optional?: boolean;
  /** One ingredient split across multiple steps ("divided"). */
  divided?: boolean;
  /** Acceptable substitutions, verbatim from the DSL. */
  substitutes?: string[];
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
  produces?: string;
  /** Numeric temperature setpoint, verbatim (e.g. "350 f"). */
  targetTemperature?: string;
  /** Stovetop heat level (e.g. "medium_high"). */
  heatLevel?: string;
  /** Structured doneness cues. */
  doneness?: UiDonenessCue[];
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
}

const DURATION_UNIT_MINUTES: Record<string, number> = {
  h: 60,
  hr: 60,
  min: 1,
  sec: 1 / 60,
  s: 1 / 60,
};
function unitToMinutes(value: string, unit: string): number {
  return Number(value) * (DURATION_UNIT_MINUTES[unit] ?? 1);
}
/** Parse a step's `duration`, supporting `N unit`, `N unit to M unit`, and
 *  `up to N unit`. Returns the lower bound (or fixed value) plus an optional
 *  upper bound. Mirrors the Rust semantic parser's duration handling. */
function parseDuration(body: string): { min: number; max?: number } {
  const upTo = body.match(/\bduration\s+up\s+to\s+([\d.]+)\s*(min|h|hr|sec|s)\b/);
  if (upTo) return { min: 0, max: unitToMinutes(upTo[1], upTo[2]) };
  const range = body.match(
    /\bduration\s+(?:estimated\s+)?([\d.]+)\s*(min|h|hr|sec|s)(?:\s+to\s+([\d.]+)\s*(min|h|hr|sec|s))?/,
  );
  if (!range) return { min: 1 };
  const min = unitToMinutes(range[1], range[2]);
  return range[3] ? { min, max: unitToMinutes(range[3], range[4]) } : { min };
}
/** Parse structured `until <kind> <value>;` doneness cues from an operation body. */
function parseDoneness(body: string): UiDonenessCue[] {
  const cues: UiDonenessCue[] = [];
  const pattern =
    /\buntil\s+(internal_temp|visual|tester|texture|rise)\s+(?:"([^"]+)"|([^;]+))\s*;/g;
  for (const match of body.matchAll(pattern)) {
    cues.push({ kind: match[1], value: (match[2] ?? match[3] ?? "").trim() });
  }
  return cues;
}

export function parseUiModel(source: string): UiRecipeModel {
  const symbol = source.match(/\brecipe\s+([A-Za-z_]\w*)/)?.[1] ?? "";
  const title = source.match(/\btitle\s+"([^"]+)"\s*;/)?.[1] ?? symbol.replaceAll("_", " ");
  const resources: UiResource[] = [];
  const pattern =
    /\b(ingredient|material|container|equipment|environment|labor|resource)\s+([A-Za-z_]\w*)(?:\s+as\s+([^\s{]+))?(?:\s+measured\s+by\s+(\w+))?\s*\{([^{}]*)\}/gms;
  for (const match of source.matchAll(pattern)) {
    const body = match[5];
    resources.push({
      kind: match[1],
      symbol: match[2],
      name: body.match(/\bname\s+"([^"]+)"\s*;/)?.[1] ?? match[2].replaceAll("_", " "),
      measurement: match[4] ?? match[3]?.match(/<([^>]+)>/)?.[1]?.toLowerCase() ?? "unspecified",
      quantity: body.match(/\b(?:quantity|mass|amount)\s+([^;]+);/)?.[1]?.trim(),
      state: body
        .match(/\bstate\s+(?:"([^"]+)"|([A-Za-z_]\w*))\s*;/)
        ?.slice(1)
        .find(Boolean),
      optional: /\boptional\s+true\s*;/.test(body) || undefined,
      divided: /\bdivided\s+true\s*;/.test(body) || undefined,
      substitutes: body
        .match(/\bsubstitutes\s+(?:\[([^\]]+)\]|([\w.]+))\s*;/)
        ?.slice(1)
        .find(Boolean)
        ?.split(",")
        .map((item) => item.trim())
        .filter(Boolean),
      range: { start: match.index ?? 0, end: (match.index ?? 0) + match[0].length },
    });
  }
  const processes = [...source.matchAll(/^\s*process\s+([A-Za-z_]\w*)/gm)].map((match) => ({
    symbol: match[1],
  }));
  const operations: UiOperation[] = [];
  let process = "root";
  const lines = source.split("\n");
  const lineOffsets: number[] = [];
  let offset = 0;
  for (const line of lines) {
    lineOffsets.push(offset);
    offset += line.length + 1;
  }
  for (let index = 0; index < lines.length; index += 1) {
    process = lines[index].match(/^\s*process\s+(\w+)/)?.[1] ?? process;
    const header = lines[index].match(/^\s*operation\s+(\w+)(?:\s+(?:does|as)\s+([^\s{]+))?/);
    if (!header) continue;
    const operationStartLine = index;
    let body = "";
    let depth = 0;
    let started = false;
    for (; index < lines.length; index += 1) {
      body += `${lines[index]}\n`;
      for (const character of lines[index]) {
        if (character === "{") {
          depth += 1;
          started = true;
        } else if (character === "}") depth -= 1;
      }
      if (started && depth === 0) break;
    }
    const duration = parseDuration(body);
    const afterText = body
      .match(/\bafter\s+(?:\[([^\]]+)\]|([\w.]+))[^;]*;/)
      ?.slice(1)
      .find(Boolean);
    // The single form may carry a per-step amount (`input butter 6 tbsp;`); we
    // still surface the resource symbol only.
    const inputText = body
      .match(/\binput\s+(?:\[([^\]]+)\]|([\w.]+)(?:\s+[\d.]+\s*\w+)?)\s*;/)
      ?.slice(1)
      .find(Boolean);
    const produces = body.match(/\bproduces\s+([\w.]+)\s*;/)?.[1];
    const temperatureMatch = body.match(/\btemperature\s+([\d.]+)\s*([A-Za-z]+)\s*;/);
    const doneness = parseDoneness(body);
    operations.push({
      inputs:
        inputText?.split(",").map((item) => item.trim().split(".").pop() ?? item.trim()) ?? [],
      produces: produces?.split(".").pop(),
      symbol: header[1],
      action: (header[2] ?? "operation").replace(/<.*$/, ""),
      process,
      durationMinutes: duration.min,
      durationMaxMinutes: duration.max,
      labor: body.match(/labor\s+(\w+)/)?.[1] ?? "unspecified",
      after: afterText?.split(",").map((item) => item.trim().split(".").pop() ?? item.trim()) ?? [],
      targetTemperature: temperatureMatch
        ? `${temperatureMatch[1]} ${temperatureMatch[2]}`
        : undefined,
      heatLevel: body.match(/\bheat\s+(low|medium_low|medium|medium_high|high)\s*;/)?.[1],
      doneness: doneness.length ? doneness : undefined,
      range: {
        start: lineOffsets[operationStartLine],
        end: lineOffsets[index] + lines[index].length,
      },
    });
  }
  // Desugar `prep <verb> <ingredient> [into <output>] (; | { ... })` into the
  // same UiOperation shape a hand-written `operation` would produce. Matching on
  // the original source keeps `range` accurate so the inspector can still edit
  // the underlying prep statement. Mirrors the Rust `prep` desugaring.
  const prepPattern =
    /\bprep\s+([A-Za-z_]\w*)\s+([A-Za-z_]\w*)(?:\s+into\s+([A-Za-z_]\w*))?\s*(\{[^{}]*\}|;)/gms;
  for (const match of source.matchAll(prepPattern)) {
    const [full, verb, ingredient, output, tail] = match;
    const producedSymbol = output ?? `${ingredient}_${verb}`;
    const body = tail.startsWith("{") ? tail : "";
    const duration = parseDuration(body);
    const afterText = body
      .match(/\bafter\s+(?:\[([^\]]+)\]|([\w.]+))[^;]*;/)
      ?.slice(1)
      .find(Boolean);
    const extraInputs =
      body
        .match(/\binput\s+(?:\[([^\]]+)\]|([\w.]+))\s*;/)
        ?.slice(1)
        .find(Boolean)
        ?.split(",")
        .map((item) => item.trim().split(".").pop() ?? item.trim()) ?? [];
    const before = source.slice(0, match.index ?? 0);
    const process = [...before.matchAll(/\bprocess\s+(\w+)/g)].pop()?.[1] ?? "root";
    operations.push({
      inputs: [ingredient, ...extraInputs],
      produces: producedSymbol,
      symbol: `${verb}_${ingredient}`,
      action: verb,
      process,
      durationMinutes: duration.min,
      durationMaxMinutes: duration.max,
      labor: body.match(/labor\s+(\w+)/)?.[1] ?? "active",
      after: afterText?.split(",").map((item) => item.trim().split(".").pop() ?? item.trim()) ?? [],
      range: { start: match.index ?? 0, end: (match.index ?? 0) + full.length },
    });
  }
  // Give every operation output that lacks a declared resource an implicit
  // intermediate material node, so the workflow graph can render it. Mirrors the
  // Rust `register_intermediates` pass.
  const declared = new Set(resources.map((resource) => resource.symbol));
  for (const operation of operations) {
    if (operation.produces && !declared.has(operation.produces)) {
      declared.add(operation.produces);
      resources.push({
        kind: "intermediate",
        symbol: operation.produces,
        name: operation.produces.replaceAll("_", " "),
        measurement: "unspecified",
      });
    }
  }
  const source_ = source.match(/\bsource\s+"([^"]+)"\s*;/)?.[1];
  const sourceUrl = source.match(/\bsource_url\s+"([^"]+)"\s*;/)?.[1];
  const attribution = source.match(/\battribution\s+"([^"]+)"\s*;/)?.[1];
  return {
    title,
    symbol,
    resources,
    processes,
    operations,
    source: source_,
    sourceUrl,
    attribution,
  };
}
