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
  range?: SourceRange;
}
export interface UiProcess {
  symbol: string;
}
export interface UiOperation {
  symbol: string;
  action: string;
  process: string;
  durationMinutes: number;
  labor: string;
  after: string[];
  range?: SourceRange;
}
export interface UiRecipeModel {
  title: string;
  symbol: string;
  resources: UiResource[];
  processes: UiProcess[];
  operations: UiOperation[];
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
    const duration = body.match(/duration\s+(?:estimated\s+)?([\d.]+)\s*(min|h|hr|sec|s)/);
    const afterText = body
      .match(/after\s+(?:\[([^\]]+)\]|([\w.]+))\s*;/)
      ?.slice(1)
      .find(Boolean);
    operations.push({
      symbol: header[1],
      action: (header[2] ?? "operation").replace(/<.*$/, ""),
      process,
      durationMinutes: duration
        ? Number(duration[1]) *
          (duration[2].startsWith("h") ? 60 : duration[2].startsWith("s") ? 1 / 60 : 1)
        : 1,
      labor: body.match(/labor\s+(\w+)/)?.[1] ?? "unspecified",
      after: afterText?.split(",").map((item) => item.trim().split(".").pop() ?? item.trim()) ?? [],
      range: {
        start: lineOffsets[operationStartLine],
        end: lineOffsets[index] + lines[index].length,
      },
    });
  }
  return { title, symbol, resources, processes, operations };
}
