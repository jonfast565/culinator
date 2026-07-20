import { computed, ref, watch, type Ref } from "vue";
import type { UiRecipeModel } from "../../recipe-editor/model";
import { setOperationPhoto } from "../../recipe-editor/sourcePatch";
import {
  emitBindings,
  emitFormula,
  emitOperation,
  emitProcess,
  emitResource,
  emitStatement,
  emitYield,
  quote,
  symbolize,
} from "../emit";
import {
  deleteDeclaration,
  duplicateDeclaration,
  editableRecipe,
  insertDeclaration,
  insertStatementAfter,
  renameSymbol,
  setDeclarationKeyword,
  setDoesVerb,
  setMeasuredBy,
  setStatement,
  setStatementList,
  swapDeclarations,
} from "../edits";
import type { Outline, OutlineNode } from "../outline";
import {
  declaredSymbols,
  findByRange,
  findStatement,
  parseOutline,
  rawValue,
  recipeNode,
  statementsWithKey,
  stringValue,
  walk,
} from "../outline";

/**
 * The editing brain of the recipe builder.
 *
 * It deliberately owns no source of truth. `source` is the editor's buffer,
 * handed in as a writable ref; every mutation writes straight back to it, which
 * trips the one watcher in `useRecipeEditor` that owns dirty state, validation,
 * and autosave. Duplicating any of that here is exactly how a form ends up
 * fighting its own autosave, so it stays a pure projection-plus-mutations layer.
 *
 * Two outlines matter. Display reads from the last one that parsed, so the form
 * keeps rendering while the user has the source temporarily unbalanced in the
 * companion editor. Editing gates on the *current* outline: a splice offset is
 * only valid for the exact text it came from, so when the live source has no
 * walkable tree (`outlineFailed`) every mutation is a no-op and the UI disables
 * its controls rather than corrupting the buffer.
 */

/** Recipe-level metadata, read for display; empty strings when absent. */
export interface BuilderMetadata {
  title: string;
  section: string;
  description: string;
  source: string;
  publisher: string;
  sourceUrl: string;
  attribution: string;
  activeTime: string;
  totalTime: string;
  coverImage: string;
}

/**
 * Metadata whose value is a bare quantity or token (`active_time 20 min;`),
 * not a quoted string. Everything else is quoted.
 */
const UNQUOTED_METADATA = new Set(["active_time", "total_time"]);

/**
 * Where recipe metadata sits, so a newly-set property is inserted after the
 * last header line already present rather than at the bottom of the block.
 */
const METADATA_ORDER = [
  "title",
  "section",
  "description",
  "source",
  "publisher",
  "source_url",
  "attribution",
  "active_time",
  "total_time",
  "image",
];

/** The declaration keywords that name a resource. */
const RESOURCE_KEYWORDS = new Set([
  "ingredient",
  "material",
  "container",
  "equipment",
  "environment",
  "labor",
]);

/**
 * Resource-block statements the builder models with a dedicated field.
 * Anything else in the block is surfaced as a read-only chip so the user can
 * see it survived, rather than silently carried through.
 */
const MODELLED_RESOURCE_KEYS = new Set([
  "name",
  "quantity",
  "state",
  "optional",
  "divided",
  "to_taste",
  "size",
  "variant",
  "substitutes",
  "note",
]);

/** A statement the builder does not model, shown verbatim. */
export interface UnknownProperty {
  keyword: string;
  text: string;
}

/** One resource, joined from the UI model and its outline node, for a card. */
export interface BuilderResource {
  symbol: string;
  kind: string;
  name: string;
  /** The `measured by` dimension, or "" when none is declared. */
  measurement: string;
  /** The declared quantity string (may be a range), or "" when none. */
  quantity: string;
  hasQuantity: boolean;
  state: string;
  size: string;
  variant: string;
  optional: boolean;
  divided: boolean;
  toTaste: boolean;
  substitutes: string[];
  notes: string[];
  unknown: UnknownProperty[];
}

/** Binding roles that name a tool or vessel rather than an input/output. */
const EQUIPMENT_ROLES = ["tool", "container", "equipment", "target"];

/**
 * Operation-block statements the builder models with a field. `requires` is
 * deliberately absent — its grammar is an unstructured token run — so it, and
 * anything else, surfaces as a read-only chip.
 */
const MODELLED_OPERATION_KEYS = new Set([
  "input",
  "output",
  "produces",
  "after",
  "duration",
  "labor",
  "temperature",
  "heat",
  "until",
  "optional",
  "repeat",
  "note",
  "photo",
  ...EQUIPMENT_ROLES,
]);

/** One input binding: a resource, optionally with a per-step amount. */
export interface BuilderBinding {
  symbol: string;
  quantity: string;
}

/** One tool/vessel binding on a step. */
export interface BuilderEquipment {
  role: string;
  symbol: string;
}

/** A structured doneness cue: `until <kind> <value>`. */
export interface BuilderDoneness {
  kind: string;
  value: string;
}

/** One step, joined from the UI model and its outline node, for a card. */
export interface BuilderOperation {
  symbol: string;
  /** The `does <verb>` action, or "" when none. */
  action: string;
  process: string;
  /** The declared duration, verbatim (`8 min`, `estimated 15 min`), or "". */
  durationText: string;
  labor: string;
  after: string[];
  inputs: BuilderBinding[];
  produces: string;
  equipment: BuilderEquipment[];
  temperature: string;
  heat: string;
  doneness: BuilderDoneness[];
  repeat: string;
  optional: boolean;
  notes: string[];
  photo: string;
  unknown: UnknownProperty[];
  /** A block-less `prep` step cannot take structured edits. */
  readOnly: boolean;
}

/** A named group of steps. */
export interface BuilderProcess {
  symbol: string;
  operations: BuilderOperation[];
}

/** A yield or serving declaration. */
export interface BuilderYield {
  keyword: string;
  symbol: string;
  measurement: string;
  amount: string;
  unknown: UnknownProperty[];
}

/** One ingredient inside a baker's formula. */
export interface BuilderFormulaIngredient {
  symbol: string;
  baker: string;
  stage: string;
}

/** A baker's-percentage formula block. */
export interface BuilderFormula {
  symbol: string;
  /** The basis clause after the symbol (`as BakersFormula`, `of total`), or "". */
  basis: string;
  target: string;
  ingredients: BuilderFormulaIngredient[];
}

/** The `measured by <dimension>` in a header, or "". */
function measuredBy(header: string): string {
  return /measured\s+by\s+(\w+)/.exec(header)?.[1] ?? "";
}

const EMPTY_OUTLINE: Outline = { nodes: [], sourceLen: 0, parsed: false };

export function useRecipeBuilder(source: Ref<string>, model: Ref<UiRecipeModel>) {
  const outline = computed(() => parseOutline(source.value));

  // The most recent outline that actually parsed. Seeded from the first
  // evaluation so display works before any edit; refreshed whenever the live
  // source parses again.
  const lastGood = ref<Outline>(outline.value.parsed ? outline.value : EMPTY_OUTLINE);
  watch(outline, (next) => {
    if (next.parsed) lastGood.value = next;
  });

  /** True when the live source cannot be walked — editing must be disabled. */
  const outlineFailed = computed(() => !outline.value.parsed);

  /** The outline to read for display: current when valid, else last good. */
  const displayOutline = computed(() => (outline.value.parsed ? outline.value : lastGood.value));

  const metadata = computed<BuilderMetadata>(() => {
    const recipe = recipeNode(displayOutline.value);
    const read = (key: string): string => {
      const node = recipe ? findStatement(recipe, key) : undefined;
      return node ? stringValue(source.value, node) : "";
    };
    // Fields the UI model already carries come from it (canonical, and correct
    // even in the recovered/degraded case); the rest are read from the outline.
    return {
      title: model.value.title,
      section: model.value.section ?? "",
      source: model.value.source ?? "",
      sourceUrl: model.value.sourceUrl ?? "",
      attribution: model.value.attribution ?? "",
      coverImage: model.value.coverImage ?? "",
      description: read("description"),
      publisher: read("publisher"),
      activeTime: read("active_time"),
      totalTime: read("total_time"),
    };
  });

  /**
   * Set, replace, or clear one recipe-level metadata property. A new property
   * is grouped with the other header lines rather than appended after the
   * ingredients and yields.
   */
  function setMetadata(key: string, rawValue: string): void {
    const recipe = editableRecipe(outline.value);
    if (!recipe) return;
    const quoted = !UNQUOTED_METADATA.has(key);
    const value = rawValue.trim();
    const existing = findStatement(recipe, key);

    if (!value) {
      if (existing) source.value = deleteDeclaration(source.value, existing);
      return;
    }
    const formatted = quoted ? quote(value) : value;
    if (existing) {
      source.value = setStatement(source.value, recipe, key, formatted);
      return;
    }
    source.value = insertMetadata(source.value, recipe, key, formatted);
  }

  function insertMetadata(
    current: string,
    recipe: NonNullable<ReturnType<typeof editableRecipe>>,
    key: string,
    formatted: string,
  ): string {
    const statement = `${key} ${formatted};`;
    // Anchor after the last header line already present (title always is, on
    // the skeleton recipe), so metadata stays clustered at the top.
    let anchor = findStatement(recipe, "title") ?? recipe.children[0];
    for (const candidate of METADATA_ORDER) {
      const node = findStatement(recipe, candidate);
      if (node) anchor = node;
    }
    if (anchor) return insertStatementAfter(current, anchor, statement);
    return setStatement(current, recipe, key, formatted);
  }

  const resources = computed<BuilderResource[]>(() => {
    // Join the UI model (always current) to the current outline for unknown
    // properties. When the live source has no walkable tree the two would be
    // from different states, so the join is skipped and chips are simply
    // omitted — editing is disabled in that state anyway.
    const active = outline.value.parsed ? outline.value : null;
    return model.value.resources
      .filter((resource) => resource.range) // only declared resources are editable
      .map((resource) => {
        const node = active && resource.range ? findByRange(active, resource.range) : undefined;
        const unknown: UnknownProperty[] = node
          ? node.children
              .filter(
                (child) => child.form === "statement" && !MODELLED_RESOURCE_KEYS.has(child.keyword),
              )
              .map((child) => ({ keyword: child.keyword, text: rawValue(source.value, child) }))
          : [];
        return {
          symbol: resource.symbol,
          kind: resource.kind,
          name: resource.name,
          measurement: resource.measurement === "unspecified" ? "" : resource.measurement,
          quantity: resource.quantity ?? "",
          hasQuantity: resource.quantity != null,
          state: resource.state ?? "",
          size: resource.size ?? "",
          variant: resource.variant ?? "",
          optional: resource.optional ?? false,
          divided: resource.divided ?? false,
          toTaste: resource.toTaste ?? false,
          substitutes: resource.substitutes ?? [],
          notes: resource.notes ?? [],
          unknown,
        };
      });
  });

  /** The current outline node for a resource, for editing. */
  function resourceNode(symbol: string): OutlineNode | undefined {
    const recipe = editableRecipe(outline.value);
    return recipe?.children.find(
      (child) =>
        child.form === "declaration" &&
        RESOURCE_KEYWORDS.has(child.keyword) &&
        child.symbol === symbol,
    );
  }

  /** A quoted string field: `name`, `state`, `size`, `variant`. */
  function setResourceString(symbol: string, key: string, value: string): void {
    const node = resourceNode(symbol);
    if (!node) return;
    const clean = value.trim();
    source.value = setStatement(source.value, node, key, clean ? quote(clean) : "");
  }

  /** The declared quantity, verbatim (`400 g`, `2 to 3 clove`); "" clears it. */
  function setResourceQuantity(symbol: string, value: string): void {
    const node = resourceNode(symbol);
    if (!node) return;
    source.value = setStatement(source.value, node, "quantity", value.trim());
  }

  /** A boolean flag: `optional`, `divided`, `to_taste`. */
  function setResourceFlag(symbol: string, key: string, on: boolean): void {
    const node = resourceNode(symbol);
    if (!node) return;
    source.value = setStatement(source.value, node, key, on ? "true" : "");
  }

  function setResourceKind(symbol: string, kind: string): void {
    const node = resourceNode(symbol);
    if (node && RESOURCE_KEYWORDS.has(kind)) {
      source.value = setDeclarationKeyword(source.value, node, kind);
    }
  }

  function setResourceMeasurement(symbol: string, dimension: string): void {
    const node = resourceNode(symbol);
    if (node) source.value = setMeasuredBy(source.value, node, dimension);
  }

  function setResourceSubstitutes(symbol: string, items: string[]): void {
    const node = resourceNode(symbol);
    if (!node) return;
    const clean = items.map((item) => item.trim()).filter(Boolean);
    source.value = setStatement(
      source.value,
      node,
      "substitutes",
      clean.length ? `[${clean.join(", ")}]` : "",
    );
  }

  function setResourceNotes(symbol: string, notes: string[]): void {
    const node = resourceNode(symbol);
    if (!node) return;
    const lines = notes
      .map((note) => note.trim())
      .filter(Boolean)
      .map((note) => emitStatement("note", quote(note)));
    source.value = setStatementList(source.value, node, "note", lines);
  }

  /** Add an empty resource of `kind`, after the last resource of any kind. */
  function addResource(kind: string): string | undefined {
    const recipe = editableRecipe(outline.value);
    if (!recipe) return undefined;
    const symbol = symbolize(`new ${kind}`, declaredSymbols(outline.value), kind);
    const hasSameKind = recipe.children.some((child) => child.keyword === kind);
    const lastResource = [...recipe.children]
      .reverse()
      .find((child) => RESOURCE_KEYWORDS.has(child.keyword));
    const afterKeyword = hasSameKind ? kind : lastResource?.keyword;
    source.value = insertDeclaration(
      source.value,
      recipe,
      emitResource({ kind, symbol }),
      afterKeyword,
    );
    return symbol;
  }

  function removeResource(symbol: string): void {
    const node = resourceNode(symbol);
    if (node) source.value = deleteDeclaration(source.value, node);
  }

  function duplicateResource(symbol: string): string | undefined {
    const node = resourceNode(symbol);
    if (!node) return undefined;
    const copy = symbolize(`${symbol} copy`, declaredSymbols(outline.value), symbol);
    source.value = duplicateDeclaration(source.value, node, copy);
    return copy;
  }

  /** Nudge a resource before/after its neighbouring resource declaration. */
  function moveResource(symbol: string, direction: "up" | "down"): void {
    const recipe = editableRecipe(outline.value);
    if (!recipe) return;
    const list = recipe.children.filter((child) => RESOURCE_KEYWORDS.has(child.keyword));
    const index = list.findIndex((child) => child.symbol === symbol);
    const target = direction === "up" ? index - 1 : index + 1;
    if (index < 0 || target < 0 || target >= list.length) return;
    const [first, second] =
      index < target ? [list[index], list[target]] : [list[target], list[index]];
    source.value = swapDeclarations(source.value, first, second);
  }

  // --- Operations & processes ------------------------------------------------

  /** Symbols the step-editor dropdowns choose from. */
  const symbols = computed(() => ({
    resources: model.value.resources.map((resource) => resource.symbol),
    operations: model.value.operations.map((operation) => operation.symbol),
  }));

  function expandEquipment(node: OutlineNode, role: string): BuilderEquipment[] {
    return statementsWithKey(node, role).flatMap((statement) => {
      const value = rawValue(source.value, statement).trim();
      if (value.startsWith("[")) {
        return value
          .slice(1, -1)
          .split(",")
          .map((item) => item.trim())
          .filter(Boolean)
          .map((symbol) => ({ role, symbol }));
      }
      // A single binding may carry an amount (`input flour 400 g`); the symbol
      // is the first token.
      return value ? [{ role, symbol: value.split(/\s+/)[0] }] : [];
    });
  }

  const processes = computed<BuilderProcess[]>(() => {
    const active = outline.value.parsed ? outline.value : null;
    const groups = new Map<string, BuilderOperation[]>();
    for (const operation of model.value.operations) {
      const node = active && operation.range ? findByRange(active, operation.range) : undefined;
      const duration = node ? findStatement(node, "duration") : undefined;
      const unknown: UnknownProperty[] = node
        ? node.children
            .filter(
              (child) => child.form === "statement" && !MODELLED_OPERATION_KEYS.has(child.keyword),
            )
            .map((child) => ({ keyword: child.keyword, text: rawValue(source.value, child) }))
        : [];
      const built: BuilderOperation = {
        symbol: operation.symbol,
        action: operation.action === "operation" ? "" : operation.action,
        process: operation.process,
        durationText: duration ? rawValue(source.value, duration) : "",
        labor: operation.labor === "unspecified" ? "" : operation.labor,
        after: operation.after,
        inputs: operation.inputBindings.map((binding) => ({
          symbol: binding.symbol,
          quantity: binding.quantity ?? "",
        })),
        produces: operation.produces ?? "",
        equipment: node ? EQUIPMENT_ROLES.flatMap((role) => expandEquipment(node, role)) : [],
        temperature: operation.targetTemperature ?? "",
        heat: operation.heatLevel ?? "",
        doneness: (operation.doneness ?? []).map((cue) => ({ kind: cue.kind, value: cue.value })),
        repeat: operation.repeat != null ? String(operation.repeat) : "",
        // The UI model does not carry an operation's `optional`, so read it off
        // the outline node directly.
        optional: node ? findStatement(node, "optional") != null : false,
        notes: operation.notes ?? [],
        photo: operation.photo ?? "",
        unknown,
        // A block-less `prep foo bar;` has no block to hold structured fields.
        readOnly: node?.keyword === "prep" && node.blockInnerRange == null,
      };
      const key = built.process || "";
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)?.push(built);
    }
    return [...groups.entries()].map(([symbol, operations]) => ({ symbol, operations }));
  });

  /** The current outline node for a step, for editing. */
  function operationNode(symbol: string): OutlineNode | undefined {
    if (!outline.value.parsed) return undefined;
    for (const node of walk(outline.value.nodes)) {
      if ((node.keyword === "operation" || node.keyword === "prep") && node.symbol === symbol) {
        return node;
      }
    }
    return undefined;
  }

  function processNode(symbol: string): OutlineNode | undefined {
    const recipe = editableRecipe(outline.value);
    return recipe?.children.find((child) => child.keyword === "process" && child.symbol === symbol);
  }

  function setOperationVerb(symbol: string, verb: string): void {
    const node = operationNode(symbol);
    if (node) source.value = setDoesVerb(source.value, node, verb);
  }

  /** Regenerate all `input` bindings for a step as one unit (see `emitBindings`). */
  function setOperationInputs(symbol: string, bindings: BuilderBinding[]): void {
    const node = operationNode(symbol);
    if (!node) return;
    const clean = bindings.filter((binding) => binding.symbol.trim());
    source.value = setStatementList(source.value, node, "input", emitBindings("input", clean));
  }

  function setOperationProduces(symbol: string, produced: string): void {
    const node = operationNode(symbol);
    if (node) source.value = setStatement(source.value, node, "produces", produced.trim());
  }

  function setOperationAfter(symbol: string, predecessors: string[]): void {
    const node = operationNode(symbol);
    if (!node) return;
    const clean = predecessors.map((item) => item.trim()).filter(Boolean);
    const line =
      clean.length === 1
        ? `after ${clean[0]};`
        : clean.length
          ? `after [${clean.join(", ")}];`
          : "";
    source.value = setStatementList(source.value, node, "after", line ? [line] : []);
  }

  /** A verbatim value field: `duration`, `labor`, `temperature`, `heat`, `repeat`. */
  function setOperationField(symbol: string, key: string, value: string): void {
    const node = operationNode(symbol);
    if (node) source.value = setStatement(source.value, node, key, value.trim());
  }

  function setOperationFlag(symbol: string, key: string, on: boolean): void {
    const node = operationNode(symbol);
    if (node) source.value = setStatement(source.value, node, key, on ? "true" : "");
  }

  function setOperationNotes(symbol: string, notes: string[]): void {
    const node = operationNode(symbol);
    if (!node) return;
    const lines = notes
      .map((note) => note.trim())
      .filter(Boolean)
      .map((note) => emitStatement("note", quote(note)));
    source.value = setStatementList(source.value, node, "note", lines);
  }

  function setOperationDoneness(symbol: string, cues: BuilderDoneness[]): void {
    const node = operationNode(symbol);
    if (!node) return;
    const lines = cues
      .filter((cue) => cue.kind.trim() && cue.value.trim())
      .map((cue) => {
        // A temperature cue is a bare quantity; the rest are quoted phrases.
        const value = cue.kind === "internal_temp" ? cue.value.trim() : quote(cue.value.trim());
        return `until ${cue.kind.trim()} ${value};`;
      });
    source.value = setStatementList(source.value, node, "until", lines);
  }

  /**
   * Replace a step's tool/container/equipment/target bindings. Each role is
   * rewritten in turn; the node is re-resolved from the freshly-reparsed source
   * between roles so every splice uses offsets valid for the text it edits.
   */
  function setOperationEquipment(symbol: string, bindings: BuilderEquipment[]): void {
    for (const role of EQUIPMENT_ROLES) {
      const node = operationNode(symbol);
      if (!node) return;
      const forRole = bindings
        .filter((binding) => binding.role === role && binding.symbol.trim())
        .map((binding) => ({ symbol: binding.symbol.trim() }));
      source.value = setStatementList(source.value, node, role, emitBindings(role, forRole));
    }
  }

  function setOperationPhotoRef(symbol: string, reference: string): void {
    const node = operationNode(symbol);
    if (node) source.value = setOperationPhoto(source.value, node.codeRange, reference);
  }

  /** Add an empty step to a process (or to the recipe when `process` is ""). */
  function addOperation(process: string): string | undefined {
    const parent = process ? processNode(process) : editableRecipe(outline.value);
    if (!parent) return undefined;
    const symbol = symbolize("step", declaredSymbols(outline.value), "step");
    const indent = `${parent.indent}    `;
    source.value = insertDeclaration(
      source.value,
      parent,
      emitOperation({ symbol, action: "prepare" }, indent),
      "operation",
    );
    return symbol;
  }

  function removeOperation(symbol: string): void {
    const node = operationNode(symbol);
    if (node) source.value = deleteDeclaration(source.value, node);
  }

  function duplicateOperation(symbol: string): string | undefined {
    const node = operationNode(symbol);
    if (!node) return undefined;
    const copy = symbolize(`${symbol} copy`, declaredSymbols(outline.value), symbol);
    source.value = duplicateDeclaration(source.value, node, copy);
    return copy;
  }

  function moveOperation(symbol: string, direction: "up" | "down"): void {
    // Reorder within the step's own process (or recipe) block.
    const owner = findOperationParent(symbol);
    if (!owner) return;
    const steps = owner.children.filter(
      (child) => child.keyword === "operation" || child.keyword === "prep",
    );
    const index = steps.findIndex((child) => child.symbol === symbol);
    const target = direction === "up" ? index - 1 : index + 1;
    if (index < 0 || target < 0 || target >= steps.length) return;
    const [first, second] =
      index < target ? [steps[index], steps[target]] : [steps[target], steps[index]];
    source.value = swapDeclarations(source.value, first, second);
  }

  /** The block (process or recipe) that directly contains this step. */
  function findOperationParent(symbol: string): OutlineNode | undefined {
    const recipe = editableRecipe(outline.value);
    if (!recipe) return undefined;
    const holds = (parent: OutlineNode): boolean =>
      parent.children.some(
        (child) =>
          (child.keyword === "operation" || child.keyword === "prep") && child.symbol === symbol,
      );
    if (holds(recipe)) return recipe;
    return recipe.children.find((child) => child.keyword === "process" && holds(child));
  }

  function addProcess(): string | undefined {
    const recipe = editableRecipe(outline.value);
    if (!recipe) return undefined;
    const symbol = symbolize("process", declaredSymbols(outline.value), "process");
    const hasProcess = recipe.children.some((child) => child.keyword === "process");
    const lastResource = [...recipe.children]
      .reverse()
      .find((child) => RESOURCE_KEYWORDS.has(child.keyword));
    const afterKeyword = hasProcess ? "process" : lastResource?.keyword;
    source.value = insertDeclaration(
      source.value,
      recipe,
      emitProcess(symbol, recipe.indent + "    "),
      afterKeyword,
    );
    return symbol;
  }

  // --- Yields & servings -----------------------------------------------------

  const yields = computed<BuilderYield[]>(() => {
    const recipe = recipeNode(displayOutline.value);
    if (!recipe) return [];
    return recipe.children
      .filter((child) => child.keyword === "yield" || child.keyword === "serving")
      .map((node) => {
        const header = node.headerRange
          ? source.value.slice(node.headerRange.start, node.headerRange.end)
          : "";
        const amount = findStatement(node, "amount");
        const unknown: UnknownProperty[] = node.children
          .filter((child) => child.form === "statement" && child.keyword !== "amount")
          .map((child) => ({ keyword: child.keyword, text: rawValue(source.value, child) }));
        return {
          keyword: node.keyword,
          symbol: node.symbol ?? "",
          measurement: measuredBy(header),
          amount: amount ? rawValue(source.value, amount) : "",
          unknown,
        };
      });
  });

  function yieldNode(symbol: string): OutlineNode | undefined {
    const recipe = editableRecipe(outline.value);
    return recipe?.children.find(
      (child) =>
        (child.keyword === "yield" || child.keyword === "serving") && child.symbol === symbol,
    );
  }

  function setYieldAmount(symbol: string, value: string): void {
    const node = yieldNode(symbol);
    if (node) source.value = setStatement(source.value, node, "amount", value.trim());
  }

  function setYieldMeasurement(symbol: string, dimension: string): void {
    const node = yieldNode(symbol);
    if (node) source.value = setMeasuredBy(source.value, node, dimension);
  }

  function addYield(keyword: string): string | undefined {
    const recipe = editableRecipe(outline.value);
    if (!recipe) return undefined;
    const base = keyword === "serving" ? "servings" : "yield";
    const symbol = symbolize(base, declaredSymbols(outline.value), base);
    const afterKeyword = recipe.children.some(
      (child) => child.keyword === "yield" || child.keyword === "serving",
    )
      ? keyword
      : undefined;
    source.value = insertDeclaration(
      source.value,
      recipe,
      emitYield(
        { keyword, symbol, measurement: "count", amount: "1 count" },
        recipe.indent + "    ",
      ),
      afterKeyword,
    );
    return symbol;
  }

  function removeYield(symbol: string): void {
    const node = yieldNode(symbol);
    if (node) source.value = deleteDeclaration(source.value, node);
  }

  // --- Formulas --------------------------------------------------------------

  const formulas = computed<BuilderFormula[]>(() => {
    const recipe = recipeNode(displayOutline.value);
    if (!recipe) return [];
    return recipe.children
      .filter((child) => child.keyword === "formula")
      .map((node) => {
        const header = node.headerRange
          ? source.value.slice(node.headerRange.start, node.headerRange.end)
          : "";
        // Everything after `formula <symbol>` is the basis clause.
        const basis = header.replace(/^\s*formula\s+\S+\s*/, "").trim();
        const target = findStatement(node, "target");
        const ingredients: BuilderFormulaIngredient[] = node.children
          .filter((child) => child.keyword === "ingredient")
          .map((child) => ({
            symbol: child.symbol ?? "",
            baker: (() => {
              const baker = findStatement(child, "baker");
              return baker ? rawValue(source.value, baker) : "";
            })(),
            stage: (() => {
              const stage = findStatement(child, "stage");
              return stage ? rawValue(source.value, stage) : "";
            })(),
          }));
        return {
          symbol: node.symbol ?? "",
          basis,
          target: target ? rawValue(source.value, target) : "",
          ingredients,
        };
      });
  });

  function formulaNode(symbol: string): OutlineNode | undefined {
    const recipe = editableRecipe(outline.value);
    return recipe?.children.find((child) => child.keyword === "formula" && child.symbol === symbol);
  }

  function setFormulaTarget(symbol: string, value: string): void {
    const node = formulaNode(symbol);
    if (node) source.value = setStatement(source.value, node, "target", value.trim());
  }

  function setFormulaIngredientBaker(formula: string, ingredient: string, value: string): void {
    const node = formulaNode(formula)?.children.find(
      (child) => child.keyword === "ingredient" && child.symbol === ingredient,
    );
    if (node) source.value = setStatement(source.value, node, "baker", value.trim());
  }

  function addFormula(): string | undefined {
    const recipe = editableRecipe(outline.value);
    if (!recipe) return undefined;
    const symbol = symbolize("formula", declaredSymbols(outline.value), "formula");
    const afterKeyword = recipe.children.some((child) => child.keyword === "formula")
      ? "formula"
      : undefined;
    source.value = insertDeclaration(
      source.value,
      recipe,
      emitFormula({ symbol, basis: "as BakersFormula", target: "1000 g" }, recipe.indent + "    "),
      afterKeyword,
    );
    return symbol;
  }

  function removeFormula(symbol: string): void {
    const node = formulaNode(symbol);
    if (node) source.value = deleteDeclaration(source.value, node);
  }

  function addFormulaIngredient(formula: string): string | undefined {
    const node = formulaNode(formula);
    if (!node) return undefined;
    const symbol = symbolize("flour", declaredSymbols(outline.value), "ingredient");
    const indent = `${node.indent}    `;
    const decl = `${indent}ingredient ${symbol} as Ingredient<BakersPercent> {\n${indent}    stage final;\n${indent}    baker 100%;\n${indent}}`;
    source.value = insertDeclaration(source.value, node, decl, "ingredient");
    return symbol;
  }

  function removeFormulaIngredient(formula: string, ingredient: string): void {
    const node = formulaNode(formula)?.children.find(
      (child) => child.keyword === "ingredient" && child.symbol === ingredient,
    );
    if (node) source.value = deleteDeclaration(source.value, node);
  }

  // --- Symbol rename ---------------------------------------------------------

  /**
   * Rename a declaration's symbol and every reference to it. The requested name
   * is normalised to a valid, unique identifier first, so the field is forgiving
   * of spaces and casing.
   */
  function renameDeclaration(from: string, to: string): void {
    if (!outline.value.parsed || !to.trim()) return;
    const taken = declaredSymbols(outline.value);
    taken.delete(from);
    const target = symbolize(to, taken, from);
    if (target && target !== from) {
      source.value = renameSymbol(source.value, outline.value, from, target);
    }
  }

  return {
    outline,
    outlineFailed,
    metadata,
    setMetadata,
    resources,
    setResourceString,
    setResourceQuantity,
    setResourceFlag,
    setResourceKind,
    setResourceMeasurement,
    setResourceSubstitutes,
    setResourceNotes,
    addResource,
    removeResource,
    duplicateResource,
    moveResource,
    processes,
    symbols,
    setOperationVerb,
    setOperationInputs,
    setOperationProduces,
    setOperationAfter,
    setOperationField,
    setOperationFlag,
    setOperationNotes,
    setOperationDoneness,
    setOperationEquipment,
    setOperationPhotoRef,
    addOperation,
    removeOperation,
    duplicateOperation,
    moveOperation,
    addProcess,
    yields,
    setYieldAmount,
    setYieldMeasurement,
    addYield,
    removeYield,
    formulas,
    setFormulaTarget,
    setFormulaIngredientBaker,
    addFormula,
    removeFormula,
    addFormulaIngredient,
    removeFormulaIngredient,
    renameDeclaration,
  };
}
