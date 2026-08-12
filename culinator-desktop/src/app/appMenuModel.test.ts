import { describe, expect, it } from "vitest";
import { buildAppMenus, formatAccelerator, type AppMenuState } from "./appMenuModel";
import { toNativeMenuSpec } from "./useNativeMenu";

function state(overrides: Partial<AppMenuState> = {}): AppMenuState {
  return {
    view: "shelf",
    hasRecipe: false,
    dirty: false,
    saving: false,
    unitSystem: "metric",
    temperatureScale: "celsius",
    misePlacement: "top-matter",
    numberStyle: "fractions",
    textSizeLabel: "A",
    bookLayout: "book",
    indexCardFormat: "full",
    recipeHeaderTypeLabel: "Medium",
    recipeBodyTypeLabel: "Medium",
    recipeAnnotationTypeLabel: "Medium",
    indexCardMarginLabel: "Medium",
    showIndexCardPager: true,
    showMenuBar: true,
    onBookView: false,
    onRecipeView: false,
    ...overrides,
  };
}

function section(menus: ReturnType<typeof buildAppMenus>, label: string) {
  const found = menus.find((menu) => menu.label === label);
  if (!found) throw new Error(`no ${label} menu`);
  return found;
}

describe("buildAppMenus", () => {
  it("disables the recipe-only sections until a recipe is open", () => {
    const closed = buildAppMenus(state());
    expect(section(closed, "Recipe").disabled).toBe(true);
    expect(section(closed, "File").disabled).toBeUndefined();

    const open = buildAppMenus(state({ hasRecipe: true, view: "reading" }));
    expect(section(open, "Recipe").disabled).toBe(false);
  });

  it("names the menu-bar toggle for what it will do", () => {
    const shown = section(buildAppMenus(state({ showMenuBar: true })), "View").items;
    const hidden = section(buildAppMenus(state({ showMenuBar: false })), "View").items;
    expect(shown.find((item) => item.action === "toggle-menu-bar")?.label).toBe("Hide menu bar");
    expect(hidden.find((item) => item.action === "toggle-menu-bar")?.label).toBe("Show menu bar");
  });

  it("gives ⌘E to whichever editor it would switch to", () => {
    const reading = section(buildAppMenus(state({ hasRecipe: true, view: "reading" })), "Recipe");
    expect(reading.items.find((item) => item.action === "build")?.accelerator).toBe("CmdOrCtrl+E");
    expect(
      reading.items.find((item) => item.action === "edit-source")?.accelerator,
    ).toBeUndefined();

    const building = section(buildAppMenus(state({ hasRecipe: true, view: "building" })), "Recipe");
    expect(building.items.find((item) => item.action === "build")?.accelerator).toBeUndefined();
    expect(building.items.find((item) => item.action === "edit-source")?.accelerator).toBe(
      "CmdOrCtrl+E",
    );
  });
});

describe("toNativeMenuSpec", () => {
  const states: AppMenuState[] = [
    state(),
    state({ hasRecipe: true, view: "reading", onRecipeView: true }),
    state({ hasRecipe: true, view: "building", onRecipeView: true, dirty: true }),
    state({ hasRecipe: true, view: "editing", onRecipeView: true, saving: true }),
    state({ view: "book", onBookView: true }),
    state({ view: "book", onBookView: true, bookLayout: "cards" }),
    state({ view: "measures" }),
  ];

  it("never registers one accelerator on two live items", () => {
    for (const current of states) {
      const enabled = toNativeMenuSpec(buildAppMenus(current))
        .flatMap((menu) => menu.items)
        .filter((item) => item.accelerator);
      const accelerators = enabled.map((item) => item.accelerator);
      expect(new Set(accelerators).size, `${current.view} / ${JSON.stringify(accelerators)}`).toBe(
        accelerators.length,
      );
    }
  });

  it("drops accelerators from items that cannot run", () => {
    // ⌘P prints the open recipe — on the shelf there is nothing to print, and a
    // key equivalent on a dead item would just swallow the keystroke.
    const shelf = toNativeMenuSpec(buildAppMenus(state()))
      .flatMap((menu) => menu.items)
      .find((item) => item.id === "print-index-card");
    expect(shelf?.enabled).toBe(false);
    expect(shelf?.accelerator).toBeUndefined();

    const reading = toNativeMenuSpec(
      buildAppMenus(state({ hasRecipe: true, view: "reading", onRecipeView: true })),
    )
      .flatMap((menu) => menu.items)
      .find((item) => item.id === "print-index-card");
    expect(reading).toMatchObject({ enabled: true, accelerator: "CmdOrCtrl+P" });
  });

  it("disables every item of a disabled section", () => {
    const recipe = toNativeMenuSpec(buildAppMenus(state())).find((menu) => menu.label === "Recipe");
    expect(recipe?.enabled).toBe(false);
    expect(recipe?.items.every((item) => !item.enabled)).toBe(true);
  });

  it("carries dividers over as separators", () => {
    const file = toNativeMenuSpec(buildAppMenus(state())).find((menu) => menu.label === "File");
    expect(file?.items.map((item) => item.separatorBefore)).toEqual([
      false,
      false,
      true,
      false,
      true,
    ]);
  });
});

describe("formatAccelerator", () => {
  it("writes macOS symbols and spelled-out modifiers elsewhere", () => {
    expect(formatAccelerator("CmdOrCtrl+S", true)).toBe("⌘S");
    expect(formatAccelerator("CmdOrCtrl+S", false)).toBe("Ctrl+S");
    expect(formatAccelerator("CmdOrCtrl+Shift+M", true)).toBe("⇧⌘M");
    expect(formatAccelerator("CmdOrCtrl+Shift+M", false)).toBe("Ctrl+Shift+M");
  });
});
