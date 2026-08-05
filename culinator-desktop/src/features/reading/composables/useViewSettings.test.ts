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

describe("book layout", () => {
  it("defaults to page-flip book and toggles to cards", () => {
    installStorage();
    const settings = useViewSettings();

    expect(settings.bookLayout.value).toBe("book");
    settings.toggleBookLayout();
    expect(settings.bookLayout.value).toBe("cards");
    settings.toggleBookLayout();
    expect(settings.bookLayout.value).toBe("book");
  });

  it("loads and persists a valid setting", async () => {
    const storage = installStorage({ "culinator.bookLayout": "cards" });
    const settings = useViewSettings();

    expect(settings.bookLayout.value).toBe("cards");
    settings.setBookLayout("book");
    await nextTick();
    expect(storage.setItem).toHaveBeenCalledWith("culinator.bookLayout", "book");
  });

  it("ignores an invalid stored setting", () => {
    installStorage({ "culinator.bookLayout": "scroll" });
    expect(useViewSettings().bookLayout.value).toBe("book");
  });
});

describe("index card format", () => {
  it("defaults to full page and persists changes", async () => {
    const storage = installStorage();
    const settings = useViewSettings();

    expect(settings.indexCardFormat.value).toBe("full");
    settings.setIndexCardFormat("6x4");
    expect(settings.indexCardFormat.value).toBe("6x4");
    await nextTick();
    expect(storage.setItem).toHaveBeenCalledWith("culinator.indexCardFormat", "6x4");
  });

  it("loads a valid stored format", () => {
    installStorage({ "culinator.indexCardFormat": "5x8" });
    expect(useViewSettings().indexCardFormat.value).toBe("5x8");
  });

  it("migrates legacy landscape 4x6 to 6x4 once", () => {
    const storage = installStorage({ "culinator.indexCardFormat": "4x6" });
    expect(useViewSettings().indexCardFormat.value).toBe("6x4");
    expect(storage.setItem).toHaveBeenCalledWith("culinator.indexCardFormat", "6x4");
    expect(storage.setItem).toHaveBeenCalledWith("culinator.indexCardFormat.migrated", "1");
  });

  it("keeps portrait 4x6 after migration flag is set", () => {
    installStorage({
      "culinator.indexCardFormat": "4x6",
      "culinator.indexCardFormat.migrated": "1",
    });
    expect(useViewSettings().indexCardFormat.value).toBe("4x6");
  });

  it("ignores an invalid stored format", () => {
    installStorage({ "culinator.indexCardFormat": "8x10" });
    expect(useViewSettings().indexCardFormat.value).toBe("full");
  });
});

describe("recipe type scales", () => {
  it("defaults to medium and cycles S → M → L", () => {
    installStorage();
    const settings = useViewSettings();
    expect(settings.recipeHeaderScale.value).toBe("md");
    settings.cycleRecipeHeaderScale();
    expect(settings.recipeHeaderScale.value).toBe("lg");
    settings.cycleRecipeHeaderScale();
    expect(settings.recipeHeaderScale.value).toBe("sm");
    settings.cycleRecipeHeaderScale();
    expect(settings.recipeHeaderScale.value).toBe("md");
  });

  it("persists body and annotation scales separately", async () => {
    const storage = installStorage({ "culinator.recipeBodyScale": "sm" });
    const settings = useViewSettings();
    expect(settings.recipeBodyScale.value).toBe("sm");
    settings.cycleRecipeAnnotationScale();
    await nextTick();
    expect(storage.setItem).toHaveBeenCalledWith("culinator.recipeAnnotationScale", "lg");
  });
});

describe("index card margins", () => {
  it("defaults to medium and cycles tight → medium → wide", () => {
    installStorage();
    const settings = useViewSettings();
    expect(settings.indexCardMargin.value).toBe("medium");
    settings.cycleIndexCardMargin();
    expect(settings.indexCardMargin.value).toBe("wide");
    settings.cycleIndexCardMargin();
    expect(settings.indexCardMargin.value).toBe("tight");
    settings.cycleIndexCardMargin();
    expect(settings.indexCardMargin.value).toBe("medium");
  });

  it("loads and persists a valid margin preset", async () => {
    const storage = installStorage({ "culinator.indexCardMargin": "tight" });
    const settings = useViewSettings();
    expect(settings.indexCardMargin.value).toBe("tight");
    settings.cycleIndexCardMargin();
    await nextTick();
    expect(storage.setItem).toHaveBeenCalledWith("culinator.indexCardMargin", "medium");
  });

  it("ignores an invalid stored margin", () => {
    installStorage({ "culinator.indexCardMargin": "huge" });
    expect(useViewSettings().indexCardMargin.value).toBe("medium");
  });
});

describe("decimal places", () => {
  it("defaults to 2 and persists changes", async () => {
    const storage = installStorage();
    const settings = useViewSettings();
    expect(settings.decimalPlaces.value).toBe(2);
    settings.setDecimalPlaces(1);
    expect(settings.decimalPlaces.value).toBe(1);
    await nextTick();
    expect(storage.setItem).toHaveBeenCalledWith("culinator.decimalPlaces", "1");
  });

  it("loads a valid stored precision", () => {
    installStorage({ "culinator.decimalPlaces": "0" });
    expect(useViewSettings().decimalPlaces.value).toBe(0);
  });

  it("ignores an invalid stored precision", () => {
    installStorage({ "culinator.decimalPlaces": "4" });
    expect(useViewSettings().decimalPlaces.value).toBe(2);
  });
});

describe("number style", () => {
  it("can be set directly for the settings dialog", async () => {
    const storage = installStorage();
    const settings = useViewSettings();
    settings.setNumberStyle("decimals");
    expect(settings.numberStyle.value).toBe("decimals");
    await nextTick();
    expect(storage.setItem).toHaveBeenCalledWith("culinator.numberStyle", "decimals");
  });
});

describe("index card pager", () => {
  it("defaults to shown and toggles off", async () => {
    const storage = installStorage();
    const settings = useViewSettings();
    expect(settings.showIndexCardPager.value).toBe(true);
    settings.toggleIndexCardPager();
    expect(settings.showIndexCardPager.value).toBe(false);
    await nextTick();
    expect(storage.setItem).toHaveBeenCalledWith("culinator.showIndexCardPager", "false");
  });

  it("loads a stored off preference", () => {
    installStorage({ "culinator.showIndexCardPager": "false" });
    expect(useViewSettings().showIndexCardPager.value).toBe(false);
  });
});
