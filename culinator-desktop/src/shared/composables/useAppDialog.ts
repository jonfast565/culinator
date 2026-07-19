import { reactive, readonly } from "vue";

export type DialogKind = "confirm" | "prompt" | "alert";

export interface DialogOptions {
  title?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  defaultValue?: string;
}

interface DialogState {
  open: boolean;
  kind: DialogKind;
  title: string;
  message: string;
  defaultValue: string;
  confirmLabel: string;
  cancelLabel: string;
}

const state = reactive<DialogState>({
  open: false,
  kind: "confirm",
  title: "",
  message: "",
  defaultValue: "",
  confirmLabel: "OK",
  cancelLabel: "Cancel",
});

let resolver: ((value: boolean | string | null) => void) | null = null;

function openDialog(
  kind: DialogKind,
  message: string,
  options: DialogOptions = {},
): Promise<boolean | string | null> {
  return new Promise((resolve) => {
    state.open = true;
    state.kind = kind;
    state.title = options.title ?? (kind === "alert" ? "Notice" : "Confirm");
    state.message = message;
    state.defaultValue = options.defaultValue ?? "";
    state.confirmLabel = options.confirmLabel ?? "OK";
    state.cancelLabel = options.cancelLabel ?? "Cancel";
    resolver = resolve;
  });
}

export function useAppDialog() {
  function confirm(message: string, options?: DialogOptions): Promise<boolean> {
    return openDialog("confirm", message, options).then((value) => value === true);
  }

  function prompt(message: string, options?: DialogOptions): Promise<string | null> {
    return openDialog("prompt", message, options).then((value) =>
      typeof value === "string" ? value : null,
    );
  }

  function alert(message: string, options?: DialogOptions): Promise<void> {
    return openDialog("alert", message, options).then(() => undefined);
  }

  function resolve(value: boolean | string | null): void {
    state.open = false;
    resolver?.(value);
    resolver = null;
  }

  return { state: readonly(state), confirm, prompt, alert, resolve };
}
