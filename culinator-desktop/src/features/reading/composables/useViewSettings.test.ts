import { nextTick } from "vue";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useViewSettings } from "./useViewSettings";

function installStorage(initial: Record<string, string> = {}) {
  const values = new Map(Object.entries(initial));
  const localStorage = {
    getItem: vi.fn((key: string) => values.get(key) ?? null),
    setItem: vi.fn((key: string, value: string) => values.set(key, value)),
  };
  vi.stubGlobal("window", { localStorage });
  return localStorage;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("reading text size", () => {
  it("defaults to the standard size and cycles through all levels", () => {
    installStorage();
    const settings = useViewSettings();

    expect(settings.textSize.value).toBe("default");
    settings.cycleTextSize();
    expect(settings.textSize.value).toBe("large");
    settings.cycleTextSize();
    expect(settings.textSize.value).toBe("x-large");
    settings.cycleTextSize();
    expect(settings.textSize.value).toBe("default");
  });

  it("loads and persists a valid setting", async () => {
    const storage = installStorage({ "culinator.textSize": "large" });
    const settings = useViewSettings();

    expect(settings.textSize.value).toBe("large");
    settings.cycleTextSize();
    await nextTick();
    expect(storage.setItem).toHaveBeenCalledWith("culinator.textSize", "x-large");
  });

  it("ignores an invalid stored setting", () => {
    installStorage({ "culinator.textSize": "enormous" });
    expect(useViewSettings().textSize.value).toBe("default");
  });
});
