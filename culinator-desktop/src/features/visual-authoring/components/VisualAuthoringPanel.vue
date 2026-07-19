<script setup lang="ts">
import { computed, ref } from "vue";
import { Plus, Trash2 } from "lucide-vue-next";
import type { UiRecipeModel, UiResource } from "../../recipe-editor/model";
import { useAppDialog } from "../../../shared/composables/useAppDialog";

const props = defineProps<{ source: string; model: UiRecipeModel }>();
const emit = defineEmits<{ "update:source": [value: string] }>();
const dialog = useAppDialog();
const selected = ref<string | null>(null);
const selectedOperation = computed(() =>
  props.model.operations.find((item) => item.symbol === selected.value),
);

// ---- Layout constants (px) --------------------------------------------------
const COL_WIDTH = 210;
const ROW_HEIGHT = 96;
const NODE_WIDTH = 168;
const NODE_HEIGHT = 62;
const HEADER_HEIGHT = 44;
const PAD = 16;

type NodeType = "operation" | "resource";
interface GraphNode {
  id: string;
  type: NodeType;
  symbol: string;
  label: string;
  sublabel: string;
  state?: string;
  labor?: string;
  kind?: string;
  level: number;
  x: number;
  y: number;
}
interface GraphEdge {
  id: string;
  from: string;
  to: string;
  kind: "data" | "control";
  path: string;
}
interface LayerSummary {
  level: number;
  x: number;
  operations: number;
  activeHands: number;
  unattended: number;
}

const LABOR_UNATTENDED = new Set(["passive", "monitor", "automated"]);

const graph = computed(() => {
  const { operations, resources } = props.model;
  const resourceBySymbol = new Map(resources.map((r) => [r.symbol, r] as const));

  // Which resources are actually wired into the workflow.
  const referenced = new Set<string>();
  for (const op of operations) {
    for (const input of op.inputs) if (resourceBySymbol.has(input)) referenced.add(input);
    if (op.produces && resourceBySymbol.has(op.produces)) referenced.add(op.produces);
  }

  const nodes = new Map<string, GraphNode>();
  for (const op of operations) {
    nodes.set(`op:${op.symbol}`, {
      id: `op:${op.symbol}`,
      type: "operation",
      symbol: op.symbol,
      label: op.action,
      sublabel: `${op.symbol} · ${round(op.durationMinutes)} min`,
      labor: op.labor,
      level: 0,
      x: 0,
      y: 0,
    });
  }
  for (const symbol of referenced) {
    const resource = resourceBySymbol.get(symbol) as UiResource;
    nodes.set(`res:${symbol}`, {
      id: `res:${symbol}`,
      type: "resource",
      symbol,
      label: resource.name || symbol.replaceAll("_", " "),
      sublabel: resource.quantity ?? resource.kind,
      state: resource.state,
      kind: resource.kind,
      level: 0,
      x: 0,
      y: 0,
    });
  }

  const edges: Omit<GraphEdge, "path">[] = [];
  for (const op of operations) {
    const opId = `op:${op.symbol}`;
    for (const input of op.inputs) {
      if (nodes.has(`res:${input}`))
        edges.push({ id: `${input}->${op.symbol}`, from: `res:${input}`, to: opId, kind: "data" });
    }
    if (op.produces && nodes.has(`res:${op.produces}`)) {
      edges.push({
        id: `${op.symbol}->${op.produces}`,
        from: opId,
        to: `res:${op.produces}`,
        kind: "data",
      });
    }
    for (const dep of op.after) {
      if (nodes.has(`op:${dep}`))
        edges.push({ id: `${dep}=>${op.symbol}`, from: `op:${dep}`, to: opId, kind: "control" });
    }
  }

  // Longest-path layering (Bellman-Ford relaxation, capped so cycles stay bounded).
  const ids = [...nodes.keys()];
  for (let pass = 0; pass < ids.length; pass += 1) {
    let changed = false;
    for (const edge of edges) {
      const from = nodes.get(edge.from);
      const to = nodes.get(edge.to);
      if (from && to && to.level < from.level + 1) {
        to.level = from.level + 1;
        changed = true;
      }
    }
    if (!changed) break;
  }

  // Assign rows within each level, preserving insertion order for stability.
  const rowByLevel = new Map<number, number>();
  let maxLevel = 0;
  let maxRows = 0;
  for (const node of nodes.values()) {
    const row = rowByLevel.get(node.level) ?? 0;
    node.x = PAD + node.level * COL_WIDTH;
    node.y = HEADER_HEIGHT + PAD + row * ROW_HEIGHT;
    rowByLevel.set(node.level, row + 1);
    maxLevel = Math.max(maxLevel, node.level);
    maxRows = Math.max(maxRows, row + 1);
  }

  const laidEdges: GraphEdge[] = edges.map((edge) => {
    const from = nodes.get(edge.from) as GraphNode;
    const to = nodes.get(edge.to) as GraphNode;
    const x1 = from.x + NODE_WIDTH;
    const y1 = from.y + NODE_HEIGHT / 2;
    const x2 = to.x;
    const y2 = to.y + NODE_HEIGHT / 2;
    const dx = Math.max(30, (x2 - x1) / 2);
    return { ...edge, path: `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}` };
  });

  // Per-layer concurrency read: operations on the same level share no path, so
  // they can run at once. Active labor needs hands; passive/monitor/automated is
  // unattended and overlaps freely.
  const layers: LayerSummary[] = [];
  for (let level = 0; level <= maxLevel; level += 1) {
    const ops = [...nodes.values()].filter((n) => n.type === "operation" && n.level === level);
    if (!ops.length) continue;
    layers.push({
      level,
      x: PAD + level * COL_WIDTH,
      operations: ops.length,
      activeHands: ops.filter((n) => n.labor === "active").length,
      unattended: ops.filter((n) => n.labor && LABOR_UNATTENDED.has(n.labor)).length,
    });
  }

  return {
    nodes: [...nodes.values()],
    edges: laidEdges,
    layers,
    width: PAD * 2 + (maxLevel + 1) * COL_WIDTH,
    height: HEADER_HEIGHT + PAD * 2 + maxRows * ROW_HEIGHT,
  };
});

const connected = computed(() => {
  const active = selected.value;
  if (!active) return new Set<string>();
  const set = new Set<string>([`op:${active}`]);
  for (const edge of graph.value.edges) {
    if (edge.from === `op:${active}`) set.add(edge.to);
    if (edge.to === `op:${active}`) set.add(edge.from);
  }
  return set;
});

function round(value: number): number {
  return Math.round(value * 10) / 10;
}
function laborColor(labor?: string): string {
  switch (labor) {
    case "active":
      return "#38634f";
    case "passive":
      return "#64748b";
    case "monitor":
      return "#b7791f";
    case "automated":
      return "#3b6ea5";
    default:
      return "#6b7280";
  }
}
function selectNode(node: GraphNode): void {
  selected.value = node.type === "operation" ? node.symbol : null;
}

// ---- Source editing (unchanged wiring) --------------------------------------
function eventValue(event: unknown): string {
  return (event as { target?: { value?: string } }).target?.value ?? "";
}
function replaceRange(start: number, end: number, text: string): void {
  emit("update:source", props.source.slice(0, start) + text + props.source.slice(end));
}
function renameTitle(event: unknown): void {
  const value = eventValue(event).replaceAll('"', '\\"');
  const regex = /\btitle\s+"[^"]*"\s*;/;
  emit(
    "update:source",
    regex.test(props.source)
      ? props.source.replace(regex, `title "${value}";`)
      : props.source.replace(/(recipe\s+\w+[^{}]*{)/, `$1\n    title "${value}";`),
  );
}
function updateOperation(field: "duration" | "labor", value: string): void {
  const operation = selectedOperation.value;
  if (!operation?.range) return;
  let block = props.source.slice(operation.range.start, operation.range.end);
  const pattern = field === "duration" ? /\bduration\s+[^;]+;/ : /\blabor\s+\w+\s*;/;
  const line = field === "duration" ? `duration ${value || "5 min"};` : `labor ${value};`;
  block = pattern.test(block)
    ? block.replace(pattern, line)
    : block.replace(/{/, `{\n            ${line}`);
  replaceRange(operation.range.start, operation.range.end, block);
}
async function deleteOperation(): Promise<void> {
  const operation = selectedOperation.value;
  if (operation?.range && (await dialog.confirm(`Delete ${operation.symbol}?`))) {
    replaceRange(operation.range.start, operation.range.end, "");
    selected.value = null;
  }
}
function addOperation(): void {
  const symbol = `operation_${props.model.operations.length + 1}`;
  const snippet = `\n    process visual_workflow {\n        operation ${symbol} does prepare {\n            duration 5 min;\n            labor active;\n        }\n    }\n`;
  emit("update:source", `${props.source.trimEnd()}\n${snippet}`);
  selected.value = symbol;
}
</script>

<template>
  <section class="panel space-y-4">
    <div>
      <label class="text-xs font-semibold uppercase tracking-wide">Recipe title</label>
      <input class="mt-1 w-full rounded border p-2" :value="model.title" @change="renameTitle" />
    </div>
    <div class="flex items-center justify-between">
      <h3>Visual workflow</h3>
      <button @click="addOperation"><Plus :size="14" /> Operation</button>
    </div>

    <p v-if="!graph.nodes.length" class="empty">
      No operations yet. Add one, or write <code>prep</code> / <code>operation</code> steps.
    </p>

    <div v-else class="graph-scroll">
      <div class="graph-canvas" :style="{ width: `${graph.width}px`, height: `${graph.height}px` }">
        <!-- Concurrency lane headers -->
        <div
          v-for="layer in graph.layers"
          :key="`layer-${layer.level}`"
          class="lane-header"
          :style="{ left: `${layer.x}px` }"
        >
          <strong>Stage {{ layer.level + 1 }}</strong>
          <small>
            {{ layer.activeHands }} hand<span v-if="layer.activeHands !== 1">s</span>
            <template v-if="layer.unattended">· {{ layer.unattended }} unattended</template>
          </small>
        </div>

        <!-- Edges -->
        <svg class="graph-edges" :width="graph.width" :height="graph.height">
          <defs>
            <marker
              id="arrow-data"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="7"
              markerHeight="7"
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z" fill="#6b8e65" />
            </marker>
            <marker
              id="arrow-control"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="7"
              markerHeight="7"
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z" fill="#b08968" />
            </marker>
          </defs>
          <path
            v-for="edge in graph.edges"
            :key="edge.id"
            :d="edge.path"
            fill="none"
            :stroke="edge.kind === 'control' ? '#b08968' : '#6b8e65'"
            :stroke-width="2"
            :stroke-dasharray="edge.kind === 'control' ? '5 4' : '0'"
            :marker-end="edge.kind === 'control' ? 'url(#arrow-control)' : 'url(#arrow-data)'"
            :opacity="!selected || connected.has(edge.from) || connected.has(edge.to) ? 1 : 0.2"
          />
        </svg>

        <!-- Nodes -->
        <button
          v-for="node in graph.nodes"
          :key="node.id"
          class="graph-node"
          :class="[
            node.type,
            {
              selected: node.type === 'operation' && node.symbol === selected,
              dim: selected && !connected.has(node.id),
            },
          ]"
          :style="{
            left: `${node.x}px`,
            top: `${node.y}px`,
            width: `${NODE_WIDTH}px`,
            height: `${NODE_HEIGHT}px`,
            borderLeftColor: node.type === 'operation' ? laborColor(node.labor) : undefined,
          }"
          @click="selectNode(node)"
        >
          <span class="node-label">
            {{ node.label }}
            <em v-if="node.state" class="state-tag">{{ node.state }}</em>
          </span>
          <span class="node-sub">{{ node.sublabel }}</span>
          <span
            v-if="node.type === 'operation'"
            class="labor-dot"
            :style="{ background: laborColor(node.labor) }"
            :title="node.labor"
          />
        </button>
      </div>
    </div>

    <!-- Legend -->
    <div class="legend">
      <span><i class="swatch line" /> data flow (produces → input)</span>
      <span><i class="swatch line dashed" /> ordering (after)</span>
      <span><i class="swatch dot" :style="{ background: '#38634f' }" /> active</span>
      <span><i class="swatch dot" :style="{ background: '#64748b' }" /> passive</span>
      <span><i class="swatch dot" :style="{ background: '#b7791f' }" /> monitor</span>
      <span><i class="swatch dot" :style="{ background: '#3b6ea5' }" /> automated</span>
    </div>

    <!-- Operation inspector -->
    <div v-if="selectedOperation" class="card space-y-3">
      <h4>{{ selectedOperation.symbol }}</h4>
      <label>
        Duration
        <input
          class="w-full rounded border p-2"
          :value="`${round(selectedOperation.durationMinutes)} min`"
          @change="updateOperation('duration', eventValue($event))"
        />
      </label>
      <label>
        Labor
        <select
          class="w-full rounded border p-2"
          :value="selectedOperation.labor"
          @change="updateOperation('labor', eventValue($event))"
        >
          <option v-for="mode in ['active', 'passive', 'monitor', 'automated']" :key="mode">
            {{ mode }}
          </option>
        </select>
      </label>
      <p class="text-xs">Inputs: {{ selectedOperation.inputs.join(", ") || "none" }}</p>
      <p v-if="selectedOperation.produces" class="text-xs">
        Produces: {{ selectedOperation.produces }}
      </p>
      <p class="text-xs">Depends on: {{ selectedOperation.after.join(", ") || "nothing" }}</p>
      <button class="danger" @click="deleteOperation">
        <Trash2 :size="14" /> Delete operation
      </button>
    </div>
  </section>
</template>

<style scoped>
.graph-scroll {
  overflow: auto;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  background:
    linear-gradient(90deg, #fbfaf6 0 1px, transparent 1px) 0 0 / 210px 100%,
    #fdfcf9;
  max-height: 60vh;
}
.graph-canvas {
  position: relative;
}
.graph-edges {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.lane-header {
  position: absolute;
  top: 8px;
  width: 168px;
  display: flex;
  flex-direction: column;
  line-height: 1.1;
  color: #6b7280;
}
.lane-header strong {
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #4b5563;
}
.lane-header small {
  font-size: 0.68rem;
}
.graph-node {
  position: absolute;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 0.15rem;
  padding: 0.4rem 0.55rem;
  border-radius: 0.5rem;
  text-align: left;
  background: #fff;
  border: 1px solid #d7dbd5;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06);
  transition: opacity 0.12s ease;
}
.graph-node.operation {
  border-left: 4px solid #38634f;
}
.graph-node.resource {
  background: #f6f1e4;
  border-color: #dcc99e;
}
.graph-node.resource.intermediate,
.graph-node.resource.material {
  background: #e9f0e8;
  border-color: #a9c3a4;
}
.graph-node.selected {
  outline: 2px solid #38634f;
  outline-offset: 1px;
}
.graph-node.dim {
  opacity: 0.35;
}
.node-label {
  font-weight: 600;
  font-size: 0.8rem;
  text-transform: capitalize;
  color: #1f2925;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.node-sub {
  font-size: 0.68rem;
  color: #6b7280;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.labor-dot {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 8px;
  height: 8px;
  border-radius: 999px;
}
.state-tag {
  margin-left: 0.25rem;
  padding: 0.02rem 0.35rem;
  border-radius: 999px;
  font-style: normal;
  font-size: 0.62rem;
  font-weight: 600;
  color: #7a5a12;
  background: #f5e6c3;
}
.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem 1rem;
  font-size: 0.7rem;
  color: #6b7280;
}
.legend span {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}
.swatch.line {
  width: 18px;
  height: 0;
  border-top: 2px solid #6b8e65;
}
.swatch.line.dashed {
  border-top: 2px dashed #b08968;
}
.swatch.dot {
  width: 9px;
  height: 9px;
  border-radius: 999px;
}
</style>
