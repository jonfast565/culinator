import { StateEffect, StateField, type Extension } from "@codemirror/state";
import { Decoration, EditorView, type DecorationSet } from "@codemirror/view";
import {
  bindingForCursor,
  definedSymbolSpans,
  type SymbolBinding,
  type SymbolKind,
  type SymbolReferenceIndex,
} from "./symbolReferences";

const setActiveBinding = StateEffect.define<SymbolBinding | null>();
const setReferenceIndex = StateEffect.define<SymbolReferenceIndex>();

const definitionMark = Decoration.mark({ class: "cm-symbol-definition" });
const useMark = Decoration.mark({ class: "cm-symbol-use" });

const kindMarks: Record<SymbolKind, ReturnType<typeof Decoration.mark>> = {
  resource: Decoration.mark({ class: "cm-recipe-symbol cm-recipe-resource" }),
  operation: Decoration.mark({ class: "cm-recipe-symbol cm-recipe-operation" }),
  process: Decoration.mark({ class: "cm-recipe-symbol cm-recipe-process" }),
  yield: Decoration.mark({ class: "cm-recipe-symbol cm-recipe-yield" }),
};

function decorationsFor(binding: SymbolBinding | null): DecorationSet {
  if (!binding) return Decoration.none;
  const ranges = [];
  if (binding.definition) {
    ranges.push(definitionMark.range(binding.definition.start, binding.definition.end));
  }
  for (const use of binding.uses) {
    ranges.push(useMark.range(use.start, use.end));
  }
  return Decoration.set(ranges, true);
}

function ambientDecorations(source: string, index: SymbolReferenceIndex): DecorationSet {
  const ranges = definedSymbolSpans(source, index).map((span) =>
    kindMarks[span.kind].range(span.start, span.end),
  );
  return ranges.length ? Decoration.set(ranges, true) : Decoration.none;
}

const activeBindingField = StateField.define<SymbolBinding | null>({
  create: () => null,
  update(value, transaction) {
    for (const effect of transaction.effects) {
      if (effect.is(setActiveBinding)) return effect.value;
    }
    return value;
  },
});

const referenceIndexField = StateField.define<SymbolReferenceIndex>({
  create: () => new Map(),
  update(value, transaction) {
    for (const effect of transaction.effects) {
      if (effect.is(setReferenceIndex)) return effect.value;
    }
    return value;
  },
});

const ambientSymbolDecorations = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, transaction) {
    for (const effect of transaction.effects) {
      if (effect.is(setReferenceIndex)) {
        return ambientDecorations(transaction.state.doc.toString(), effect.value);
      }
    }
    if (transaction.docChanged) return value.map(transaction.changes);
    return value;
  },
  provide: (field) => EditorView.decorations.from(field),
});

const bindingDecorations = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, transaction) {
    for (const effect of transaction.effects) {
      if (effect.is(setActiveBinding)) return decorationsFor(effect.value);
    }
    if (transaction.docChanged) return value.map(transaction.changes);
    return value;
  },
  provide: (field) => EditorView.decorations.from(field),
});

function syncActiveBinding(view: EditorView): void {
  const index = view.state.field(referenceIndexField);
  const pos = view.state.selection.main.head;
  const source = view.state.doc.toString();
  const next = bindingForCursor(index, source, pos);
  const current = view.state.field(activeBindingField);
  if (current?.symbol === next?.symbol) return;
  view.dispatch({ effects: setActiveBinding.of(next) });
}

const referenceNavigationTheme = EditorView.theme({
  ".cm-recipe-symbol": {
    fontWeight: "600",
  },
  ".cm-recipe-resource, .cm-recipe-resource span": {
    color: "#0b6e99",
  },
  ".cm-recipe-operation, .cm-recipe-operation span": {
    color: "#6b3fa0",
  },
  ".cm-recipe-process, .cm-recipe-process span": {
    color: "#8a4b12",
  },
  ".cm-recipe-yield, .cm-recipe-yield span": {
    color: "#0f6b5c",
  },
  ".cm-symbol-definition": {
    backgroundColor: "rgb(40 100 59 / 22%)",
    borderRadius: "3px",
    outline: "1px solid rgb(40 100 59 / 45%)",
  },
  ".cm-symbol-use": {
    backgroundColor: "rgb(184 134 11 / 20%)",
    borderRadius: "3px",
    outline: "1px solid rgb(184 134 11 / 40%)",
  },
});

/**
 * Highlight the binding under the cursor and jump to its definition with
 * Ctrl/Cmd-click — the DrRacket "see where this name is bound" affordance.
 * Recipe-defined names also stay permanently colored by kind.
 */
export function referenceNavigation(): Extension {
  return [
    referenceIndexField,
    activeBindingField,
    ambientSymbolDecorations,
    bindingDecorations,
    referenceNavigationTheme,
    EditorView.updateListener.of((update) => {
      if (update.selectionSet || update.docChanged) syncActiveBinding(update.view);
    }),
    EditorView.domEventHandlers({
      click(event, view) {
        if (!(event.metaKey || event.ctrlKey)) return false;
        const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
        if (pos == null) return false;
        const binding = bindingForCursor(
          view.state.field(referenceIndexField),
          view.state.doc.toString(),
          pos,
        );
        const target = binding?.definition;
        if (!target) return false;
        event.preventDefault();
        view.dispatch({
          selection: { anchor: target.start, head: target.end },
          effects: [
            setActiveBinding.of(binding),
            EditorView.scrollIntoView(target.start, { y: "center" }),
          ],
        });
        return true;
      },
    }),
  ];
}

/** Keep the live reference index in sync as the recipe model reparses. */
export function setEditorReferenceIndex(view: EditorView, index: SymbolReferenceIndex): void {
  view.dispatch({ effects: setReferenceIndex.of(index) });
  syncActiveBinding(view);
}
