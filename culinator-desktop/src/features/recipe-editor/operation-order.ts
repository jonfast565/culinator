import type { UiOperation } from "./model";

/** Sort key for parallel operations at the same dependency level. */
function sourceOrder(a: UiOperation, b: UiOperation): number {
  return (a.range?.start ?? 0) - (b.range?.start ?? 0);
}

/** Topological sort on `after` deps; parallel steps tie-break by source position. */
export function sortOperationsForDisplay(ops: UiOperation[]): UiOperation[] {
  if (ops.length <= 1) return [...ops];

  const symbols = new Set(ops.map((operation) => operation.symbol));
  const placed = new Set<string>();
  const result: UiOperation[] = [];

  while (result.length < ops.length) {
    const ready = ops.filter(
      (operation) =>
        !placed.has(operation.symbol) &&
        operation.after.every((dependency) => !symbols.has(dependency) || placed.has(dependency)),
    );
    if (!ready.length) {
      const remaining = ops.filter((operation) => !placed.has(operation.symbol)).sort(sourceOrder);
      result.push(...remaining);
      break;
    }
    ready.sort(sourceOrder);
    for (const operation of ready) {
      placed.add(operation.symbol);
      result.push(operation);
    }
  }

  return result;
}
