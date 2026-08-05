import { describe, expect, it } from "vitest";
import type { IngredientGroup, MethodSection } from "../recipe-editor/narrative";
import { INDEX_CARD_SPECS, type IndexCardFormat } from "./indexCardFormat";
import type { IndexCardMargin } from "./indexCardMargin";
import type { RecipeTypeScale } from "./recipeTypeScale";
import {
  cardContentBox,
  chunkIngredientGroups,
  estimateIndexCardPageHeight,
  indexCardIngredientsSectionLabel,
  indexCardSectionLabel,
  paginateIndexCards,
} from "./paginateIndexCards";

const ALL_CARD_FORMATS = Object.keys(INDEX_CARD_SPECS) as Exclude<IndexCardFormat, "full">[];
const ALL_MARGINS: IndexCardMargin[] = ["tight", "medium", "wide"];
const TYPE_SCALES: RecipeTypeScale[] = ["sm", "md", "lg"];

function countPlacedIngredients(pages: ReturnType<typeof paginateIndexCards>): number {
  return pages.reduce(
    (sum, page) =>
      sum + page.ingredientGroups.reduce((n, group) => n + group.items.length, 0),
    0,
  );
}

function countPlacedSteps(pages: ReturnType<typeof paginateIndexCards>): number {
  return pages.reduce(
    (sum, page) =>
      sum + page.methodBlocks.reduce((n, block) => n + block.steps.length, 0),
    0,
  );
}

/** Minimum slack (px) between estimate and card height — catches optimistic packing. */
const PAGE_HEIGHT_SLACK_PX = 2;
/** Method pages need extra headroom — prose wraps unpredictably at narrow widths. */
const METHOD_PAGE_SLACK_PX = 4;

function item(
  symbol: string,
  description: string,
  amount = "1",
  aside?: string,
): IngredientGroup["items"][number] {
  return { symbol, amount, description, aside };
}

function step(symbol: string, number: number, text: string) {
  return { symbol, number, text, tools: [] as string[] };
}

function brigadeiroSections(): MethodSection[] {
  return [
    {
      process: "cooking",
      title: "Cooking",
      steps: [
        step("s1", 1, "Combine condensed milk, chocolate, and butter."),
        step("s2", 2, "Cook until thick."),
      ],
      mise: { ingredients: [], equipment: [] },
    },
    {
      process: "finishing",
      title: "Finishing",
      steps: [step("s3", 3, "Cool, roll into balls, and coat in sprinkles.")],
      mise: { ingredients: [], equipment: [] },
    },
  ];
}

const shortIngredients: IngredientGroup[] = [
  {
    items: [
      item("a", "condensed milk"),
      item("b", "chocolate"),
      item("c", "butter"),
      item("d", "sprinkles"),
    ],
  },
];

/** Narrative-shaped Apple Pie ingredient list (16 items, egg-wash variant group). */
function applePieIngredientGroups(): IngredientGroup[] {
  return [
    {
      items: [
        item("apples", "golden delicious apples", "900 g", "4-5 apples"),
        item("lemon_juice", "lemon juice", "15 g"),
        item("filling_white_sugar", "white sugar", "50 g"),
        item("brown_sugar", "brown sugar", "65 g"),
        item("cinnamon", "cinnamon", "5 g"),
        item("nutmeg", "nutmeg", "1 g to 2 g"),
        item("cornstarch", "cornstarch", "15 g"),
        item("crust_flour", "flour", "350 g"),
        item("crust_salt", "salt", "5 g"),
        item("crust_sugar", "white sugar", "30 g"),
        item("crust_butter", "cold butter", "225 g", "cut into slices"),
        item("ice_water", "ice water", "50 ml to 100 ml"),
      ],
    },
    {
      label: "Egg wash",
      items: [
        item("egg", "egg", "1 count", "optional"),
        item("wash_water", "water", "15 g", "optional"),
      ],
    },
  ];
}

function applePieEquipment(): string[] {
  return [
    "mixing bowl",
    "saucepan",
    "9-inch pie pan",
    "food processor",
    "rolling pin",
    "knife",
    "pastry brush",
    "oven",
  ];
}

function applePieSections(): MethodSection[] {
  return [
    {
      process: "filling",
      title: "Filling",
      steps: [
        step("cut", 1, "Peel and cut the apples into strips."),
        step("macerate", 2, "Toss with sugar and spices; rest."),
      ],
      mise: { ingredients: [], equipment: [] },
    },
    {
      process: "crust",
      title: "Crust",
      steps: [
        step("mix", 3, "Cut butter into flour until sandy."),
        step("roll", 4, "Roll the dough and line the pan."),
      ],
      mise: { ingredients: [], equipment: [] },
    },
  ];
}

describe("chunkIngredientGroups", () => {
  it("splits long lists while preserving group labels", () => {
    const groups: IngredientGroup[] = [
      {
        label: "Sweet",
        items: [item("a", "A"), item("b", "B"), item("c", "C"), item("d", "D")],
      },
    ];
    const slices = chunkIngredientGroups(groups, 3);
    expect(slices).toHaveLength(2);
    expect(slices[0][0].label).toBe("Sweet");
    expect(slices[0][0].items).toHaveLength(3);
    expect(slices[1][0].label).toBe("Sweet (cont.)");
    expect(slices[1][0].items).toHaveLength(1);
  });
});

describe("cardContentBox", () => {
  it("gives Small type and tight margins more usable height than Large/wide", () => {
    const tightSmall = cardContentBox("6x4", "tight", { body: "sm" });
    const wideLarge = cardContentBox("6x4", "wide", { body: "lg" });
    expect(tightSmall.height).toBeGreaterThan(wideLarge.height);
    expect(tightSmall.body).toBeLessThan(wideLarge.body);
  });
});

describe("indexCardSectionLabel", () => {
  it("marks equipment and method as continued without title or ingredients on the card", () => {
    expect(
      indexCardSectionLabel("Equipment", {
        showTitle: false,
        ingredientGroups: [],
      }),
    ).toBe("Equipment (cont.)");
    expect(
      indexCardSectionLabel("Method", {
        showTitle: true,
        ingredientGroups: [],
      }),
    ).toBe("Method");
    expect(
      indexCardSectionLabel("Equipment", {
        showTitle: false,
        ingredientGroups: [{ items: [item("a", "flour")] }],
      }),
    ).toBe("Equipment");
  });
});

describe("paginateIndexCards continuation labels", () => {
  it("labels equipment (cont.) when it spills to a continuation-only card", () => {
    const equipment = Array.from(
      { length: 20 },
      (_, index) => `Tool number ${index + 1} with a descriptive name`,
    );
    const pages = paginateIndexCards({
      format: "3x5",
      bodyScale: "lg",
      margin: "wide",
      ingredientGroups: [{ items: [item("a", "flour"), item("b", "water")] }],
      equipment,
      sections: [
        {
          process: "main",
          steps: [step("s1", 1, "Combine and cook.")],
          mise: { ingredients: [], equipment: [] },
        },
      ],
    });
    const equipmentOnlyPages = pages.filter(
      (page) =>
        page.equipment.length > 0 && !page.showTitle && !page.ingredientGroups.length,
    );
    expect(equipmentOnlyPages.length).toBeGreaterThan(0);
    for (const page of equipmentOnlyPages) {
      expect(indexCardSectionLabel("Equipment", page)).toBe("Equipment (cont.)");
    }
  });

  it("keeps Apple Pie-scale cards coherent with continuation labels and order", () => {
    const pages = paginateIndexCards({
      format: "6x4",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups: applePieIngredientGroups(),
      equipment: applePieEquipment(),
      sections: applePieSections(),
    });
    const m = cardContentBox("6x4", "medium", { body: "md", header: "md", annotation: "md" });
    const equipmentPageIndex = pages.findIndex((page) => page.equipment.length > 0);
    const firstMethodPageIndex = pages.findIndex((page) => page.methodBlocks.length > 0);
    expect(equipmentPageIndex).toBeGreaterThanOrEqual(0);
    expect(firstMethodPageIndex).toBeGreaterThanOrEqual(0);
    expect(equipmentPageIndex).toBeLessThanOrEqual(firstMethodPageIndex);

    for (const page of pages) {
      expect(estimateIndexCardPageHeight(page, m)).toBeLessThanOrEqual(m.height);
      if (page.ingredientGroups.length) {
        expect(indexCardIngredientsSectionLabel(page)).toBe(
          page.showTitle ? "Ingredients" : "Ingredients (cont.)",
        );
      }
      if (page.equipment.length) {
        expect(indexCardSectionLabel("Equipment", page)).toBe(
          page.showTitle || page.ingredientGroups.length ? "Equipment" : "Equipment (cont.)",
        );
      }
      if (page.methodBlocks.length && !page.showTitle && !page.ingredientGroups.length) {
        expect(indexCardSectionLabel("Method", page)).toBe("Method (cont.)");
      }
    }
  });

  it("labels a split method section with (cont.) on the continuation card", () => {
    const pages = paginateIndexCards({
      format: "3x5",
      bodyScale: "lg",
      margin: "wide",
      ingredientGroups: [{ items: [item("a", "flour"), item("b", "water")] }],
      equipment: [],
      sections: [
        {
          process: "main",
          title: "Cooking",
          steps: Array.from({ length: 6 }, (_, index) =>
            step(
              `s${index}`,
              index + 1,
              `Carry out step ${index + 1} carefully, watching the pan and adjusting heat as needed for even browning.`,
            ),
          ),
          mise: { ingredients: [], equipment: [] },
        },
      ],
    });
    expect(pages.length).toBeGreaterThan(1);
    const continuedBlock = pages
      .flatMap((page) => page.methodBlocks)
      .find((block) => block.title?.endsWith("(cont.)"));
    expect(continuedBlock?.title).toBe("Cooking (cont.)");
  });
});

describe("paginateIndexCards", () => {
  it("puts leftover content on later cards when the front card fills up", () => {
    const ingredientGroups: IngredientGroup[] = [
      {
        items: Array.from({ length: 18 }, (_, index) =>
          item(
            `i${index}`,
            `Chopped ingredient number ${index} with a longer description line`,
          ),
        ),
      },
    ];
    const sections: MethodSection[] = [
      {
        process: "main",
        steps: Array.from({ length: 6 }, (_, index) =>
          step(
            `s${index}`,
            index + 1,
            `Carry out step ${index + 1} carefully, watching the pan and adjusting heat as needed.`,
          ),
        ),
        mise: { ingredients: [], equipment: [] },
      },
    ];
    const pages = paginateIndexCards({
      format: "3x5",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups,
      equipment: ["Bowl"],
      sections,
    });
    expect(pages[0].showTitle).toBe(true);
    expect(pages.some((page) => page.equipment.includes("Bowl"))).toBe(true);
    expect(pages.length).toBeGreaterThan(1);
    expect(pages.some((page) => page.methodBlocks.length > 0)).toBe(true);
    expect(pages.every((page) => page.total === pages.length)).toBe(true);
    const placedIngredients = pages.reduce(
      (sum, page) =>
        sum + page.ingredientGroups.reduce((n, group) => n + group.items.length, 0),
      0,
    );
    expect(placedIngredients).toBe(18);
  });

  it("colocates method on the front card when the ingredient list is short", () => {
    const pages = paginateIndexCards({
      format: "6x4",
      ingredientGroups: [{ items: [item("butter", "butter")] }],
      equipment: [],
      sections: [
        {
          process: "main",
          steps: [step("melt", 1, "Melt the butter.")],
          mise: { ingredients: [], equipment: [] },
        },
      ],
    });
    expect(pages).toHaveLength(1);
    expect(pages[0].showTitle).toBe(true);
    expect(pages[0].ingredientGroups[0].items).toHaveLength(1);
    expect(pages[0].methodBlocks[0].steps).toHaveLength(1);
  });

  it("keeps a short 4×6 recipe on one card at Medium type", () => {
    const pages = paginateIndexCards({
      format: "6x4",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups: shortIngredients,
      equipment: ["saucepan"],
      sections: brigadeiroSections(),
    });
    expect(pages).toHaveLength(1);
    expect(pages[0].ingredientGroups[0].items).toHaveLength(4);
    const steps = pages[0].methodBlocks.flatMap((block) => block.steps);
    expect(steps).toHaveLength(3);
  });

  it("allows Small type / tight margins to pack denser than Large / wide", () => {
    const longSteps: MethodSection[] = [
      {
        process: "main",
        steps: Array.from({ length: 8 }, (_, index) =>
          step(
            `s${index}`,
            index + 1,
            `This is a moderately long instruction number ${index + 1} that wraps on a card.`,
          ),
        ),
        mise: { ingredients: [], equipment: [] },
      },
    ];
    const dense = paginateIndexCards({
      format: "6x4",
      bodyScale: "sm",
      margin: "tight",
      ingredientGroups: shortIngredients,
      equipment: [],
      sections: longSteps,
    });
    const roomy = paginateIndexCards({
      format: "6x4",
      bodyScale: "lg",
      margin: "wide",
      ingredientGroups: shortIngredients,
      equipment: [],
      sections: longSteps,
    });
    expect(dense.length).toBeLessThanOrEqual(roomy.length);
    expect(roomy.length).toBeGreaterThan(1);
  });

  it("never drops ingredients from a long top-matter list (Apple Pie scale)", () => {
    const ingredientGroups = applePieIngredientGroups();
    const equipment = applePieEquipment();
    const pages = paginateIndexCards({
      format: "6x4",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups,
      equipment,
      sections: applePieSections(),
    });
    const m = cardContentBox("6x4", "medium", { body: "md", header: "md", annotation: "md" });
    const placed = pages.reduce(
      (sum, page) =>
        sum + page.ingredientGroups.reduce((n, group) => n + group.items.length, 0),
      0,
    );
    expect(placed).toBe(14);
    expect(pages.length).toBeGreaterThan(1);
    for (const page of pages) {
      expect(estimateIndexCardPageHeight(page, m)).toBeLessThanOrEqual(m.height);
    }
    const equipmentPageIndex = pages.findIndex((page) => page.equipment.length > 0);
    const firstMethodPageIndex = pages.findIndex((page) => page.methodBlocks.length > 0);
    expect(equipmentPageIndex).toBeGreaterThanOrEqual(0);
    expect(firstMethodPageIndex).toBeGreaterThanOrEqual(0);
    expect(equipmentPageIndex).toBeLessThanOrEqual(firstMethodPageIndex);
    expect(pages[equipmentPageIndex].equipment).toEqual(equipment);
    const trailingEquipmentOnly = pages.some(
      (page, index) =>
        page.equipment.length > 0 &&
        page.methodBlocks.length === 0 &&
        pages.slice(0, index).some((prior) => prior.methodBlocks.length > 0),
    );
    expect(trailingEquipmentOnly).toBe(false);
  });

  it("keeps realistic Apple Pie ingredients within the medium-margin height budget", () => {
    const ingredientGroups = applePieIngredientGroups();
    const pages = paginateIndexCards({
      format: "6x4",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups,
      equipment: applePieEquipment(),
      sections: applePieSections(),
    });
    const m = cardContentBox("6x4", "medium", { body: "md", header: "md", annotation: "md" });
    const ingredientPages = pages.filter((page) => page.ingredientGroups.length > 0);

    expect(ingredientPages.length).toBeGreaterThan(1);
    for (const page of ingredientPages) {
      const estimated = estimateIndexCardPageHeight(page, m);
      expect(estimated).toBeLessThanOrEqual(m.height);
      expect(m.height - estimated).toBeGreaterThanOrEqual(PAGE_HEIGHT_SLACK_PX);
    }

    const tightPages = paginateIndexCards({
      format: "6x4",
      bodyScale: "md",
      margin: "tight",
      ingredientGroups,
      equipment: [],
      sections: [],
    });
    const firstMediumCount = pages[0].ingredientGroups.reduce(
      (sum, group) => sum + group.items.length,
      0,
    );
    const firstTightCount = tightPages[0].ingredientGroups.reduce(
      (sum, group) => sum + group.items.length,
      0,
    );
    expect(firstMediumCount).toBeLessThanOrEqual(firstTightCount);
  });

  it("packs Apple Pie ingredients densely on large card formats at Large type", () => {
    for (const format of ["5x8", "8x5"] as const) {
      const pages = paginateIndexCards({
        format,
        bodyScale: "lg",
        headerScale: "lg",
        annotationScale: "lg",
        margin: "medium",
        ingredientGroups: applePieIngredientGroups(),
        equipment: applePieEquipment(),
        sections: applePieSections(),
      });
      const m = cardContentBox(format, "medium", { body: "lg", header: "lg", annotation: "lg" });
      const ingredientPages = pages.filter((page) => page.ingredientGroups.length > 0);

      expect(ingredientPages.length).toBeLessThanOrEqual(2);
      if (format === "5x8") {
        expect(ingredientPages).toHaveLength(1);
        expect(countPlacedIngredients(pages)).toBe(14);
      }
      if (format === "8x5") {
        const firstCount = ingredientPages[0].ingredientGroups.reduce(
          (sum, group) => sum + group.items.length,
          0,
        );
        expect(firstCount).toBeGreaterThanOrEqual(13);
      }

      for (const page of pages) {
        expect(estimateIndexCardPageHeight(page, m)).toBeLessThanOrEqual(m.height);
      }
    }

    const mediumLandscape = paginateIndexCards({
      format: "8x5",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups: applePieIngredientGroups(),
      equipment: applePieEquipment(),
      sections: applePieSections(),
    });
    expect(
      mediumLandscape.filter((page) => page.ingredientGroups.length > 0),
    ).toHaveLength(1);
  });

  it("places equipment on the first card before method for a short recipe", () => {
    const pages = paginateIndexCards({
      format: "6x4",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups: shortIngredients,
      equipment: ["saucepan"],
      sections: brigadeiroSections(),
    });
    expect(pages).toHaveLength(1);
    expect(pages[0].equipment).toEqual(["saucepan"]);
    expect(pages[0].ingredientGroups[0].items).toHaveLength(4);
    expect(pages[0].methodBlocks.flatMap((block) => block.steps)).toHaveLength(3);
  });

  it("places equipment on the ingredient-complete card, not after all method steps", () => {
    const ingredientGroups: IngredientGroup[] = [
      {
        items: Array.from({ length: 18 }, (_, index) =>
          item(
            `i${index}`,
            `Chopped ingredient number ${index} with a longer description line`,
          ),
        ),
      },
    ];
    const equipment = ["Bowl", "Whisk", "Skillet"];
    const pages = paginateIndexCards({
      format: "3x5",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups,
      equipment,
      sections: [
        {
          process: "main",
          steps: Array.from({ length: 6 }, (_, index) =>
            step(
              `s${index}`,
              index + 1,
              `Carry out step ${index + 1} carefully, watching the pan and adjusting heat as needed.`,
            ),
          ),
          mise: { ingredients: [], equipment: [] },
        },
      ],
    });
    const equipmentPageIndex = pages.findIndex((page) => page.equipment.length > 0);
    const firstMethodPageIndex = pages.findIndex((page) => page.methodBlocks.length > 0);
    const lastMethodPageIndex = pages.reduce(
      (last, page, index) => (page.methodBlocks.length > 0 ? index : last),
      -1,
    );
    expect(equipmentPageIndex).toBeGreaterThanOrEqual(0);
    expect(firstMethodPageIndex).toBeGreaterThanOrEqual(0);
    expect(equipmentPageIndex).toBeLessThanOrEqual(firstMethodPageIndex);
    expect(equipmentPageIndex).toBeLessThan(lastMethodPageIndex);
    expect(pages[equipmentPageIndex].equipment).toEqual(equipment);
    const trailingEquipmentOnly = pages.some(
      (page, index) =>
        page.equipment.length > 0 &&
        page.methodBlocks.length === 0 &&
        pages.slice(0, index).some((prior) => prior.methodBlocks.length > 0),
    );
    expect(trailingEquipmentOnly).toBe(false);
  });

  it("switches to per-section mise without a top ingredient list", () => {
    const pages = paginateIndexCards({
      format: "6x4",
      miseLayout: "colocated",
      ingredientGroups: shortIngredients,
      equipment: ["saucepan"],
      sections: [
        {
          process: "cooking",
          title: "Cooking",
          steps: [step("s1", 1, "Combine condensed milk, chocolate, and butter.")],
          mise: {
            ingredients: [
              item("a", "condensed milk"),
              item("b", "chocolate"),
              item("c", "butter"),
            ],
            equipment: ["saucepan"],
          },
        },
        {
          process: "finishing",
          title: "Finishing",
          steps: [step("s2", 2, "Cool and roll in sprinkles.")],
          mise: {
            ingredients: [item("d", "sprinkles")],
            equipment: [],
          },
        },
      ],
    });
    expect(pages.every((page) => page.ingredientGroups.length === 0)).toBe(true);
    expect(pages.every((page) => page.equipment.length === 0)).toBe(true);
    const mises = pages.flatMap((page) =>
      page.methodBlocks.filter((block) => block.mise).map((block) => block.mise!),
    );
    expect(mises.length).toBeGreaterThanOrEqual(1);
    expect(mises.some((mise) => mise.ingredients.some((item) => item.symbol === "a"))).toBe(
      true,
    );
  });

  it("splits very long method prose across cards even with few steps", () => {
    const pages = paginateIndexCards({
      format: "3x5",
      bodyScale: "lg",
      margin: "wide",
      ingredientGroups: [{ items: [item("a", "flour"), item("b", "water")] }],
      equipment: [],
      sections: [
        {
          process: "main",
          steps: [
            step(
              "s1",
              1,
              "Whisk the flour and water together until completely smooth, scraping the sides of the bowl, then rest the batter for ten minutes before using.",
            ),
            step(
              "s2",
              2,
              "Heat a thin film of oil in a heavy skillet over medium heat, pour a ladle of batter, swirl to coat, and cook until the edges lift cleanly from the pan.",
            ),
            step(
              "s3",
              3,
              "Flip carefully with a wide spatula and cook the second side until spotted brown; transfer to a plate and repeat with the remaining batter.",
            ),
            step(
              "s4",
              4,
              "Stack the finished rounds under a clean towel so they stay warm and pliable while you finish the rest of the batch for serving.",
            ),
          ],
          mise: { ingredients: [], equipment: [] },
        },
      ],
    });
    expect(pages.length).toBeGreaterThan(1);
    const totalSteps = pages.reduce(
      (sum, page) =>
        sum + page.methodBlocks.reduce((n, block) => n + block.steps.length, 0),
      0,
    );
    expect(totalSteps).toBe(4);
  });
});

describe("index card height budget (all presets)", () => {
  const presetMatrix = ALL_CARD_FORMATS.flatMap((format) =>
    ALL_MARGINS.map((margin) => ({ format, margin })),
  );

  it.each(presetMatrix)(
    "packs Apple Pie-scale content within budget for $format / $margin",
    ({ format, margin }) => {
      const pages = paginateIndexCards({
        format,
        bodyScale: "md",
        margin,
        ingredientGroups: applePieIngredientGroups(),
        equipment: applePieEquipment(),
        sections: applePieSections(),
      });
      const m = cardContentBox(format, margin, { body: "md", header: "md", annotation: "md" });

      expect(countPlacedIngredients(pages)).toBe(14);
      expect(countPlacedSteps(pages)).toBe(4);

      for (const page of pages) {
        const estimated = estimateIndexCardPageHeight(page, m);
        expect(estimated, JSON.stringify({ format, margin, page: page.index })).toBeLessThanOrEqual(
          m.height,
        );
        if (page.methodBlocks.length) {
          expect(m.height - estimated).toBeGreaterThanOrEqual(METHOD_PAGE_SLACK_PX);
        }
      }
    },
  );

  it.each(presetMatrix)(
    "packs long method prose within budget for $format / $margin",
    ({ format, margin }) => {
      const longSteps: MethodSection[] = [
        {
          process: "main",
          steps: Array.from({ length: 5 }, (_, index) =>
            step(
              `s${index}`,
              index + 1,
              `Carry out step ${index + 1} carefully, watching the pan and adjusting heat as needed for even browning throughout.`,
            ),
          ),
          mise: { ingredients: [], equipment: [] },
        },
      ];
      const pages = paginateIndexCards({
        format,
        bodyScale: "md",
        margin,
        ingredientGroups: [{ items: [item("a", "flour"), item("b", "water")] }],
        equipment: [],
        sections: longSteps,
      });
      const m = cardContentBox(format, margin, { body: "md", header: "md", annotation: "md" });

      expect(countPlacedSteps(pages)).toBe(5);
      for (const page of pages) {
        const estimated = estimateIndexCardPageHeight(page, m);
        expect(estimated).toBeLessThanOrEqual(m.height);
        if (page.methodBlocks.length) {
          expect(m.height - estimated).toBeGreaterThanOrEqual(METHOD_PAGE_SLACK_PX);
        }
      }
    },
  );

  it.each(TYPE_SCALES)(
    "packs brigadeiro-scale short recipe within budget at 6x4/medium scale=%s",
    (bodyScale) => {
      const pages = paginateIndexCards({
        format: "6x4",
        bodyScale,
        margin: "medium",
        headerScale: bodyScale,
        annotationScale: bodyScale,
        ingredientGroups: shortIngredients,
        equipment: ["saucepan"],
        sections: brigadeiroSections(),
      });
      const m = cardContentBox("6x4", "medium", {
        body: bodyScale,
        header: bodyScale,
        annotation: bodyScale,
      });

      expect(countPlacedIngredients(pages)).toBe(4);
      expect(countPlacedSteps(pages)).toBe(3);
      for (const page of pages) {
        const estimated = estimateIndexCardPageHeight(page, m);
        expect(estimated).toBeLessThanOrEqual(m.height);
        expect(m.height - estimated).toBeGreaterThanOrEqual(PAGE_HEIGHT_SLACK_PX);
      }
    },
  );

  it("keeps brigadeiro on one 6x4 card at medium margin and type", () => {
    const pages = paginateIndexCards({
      format: "6x4",
      bodyScale: "md",
      margin: "medium",
      ingredientGroups: shortIngredients,
      equipment: ["saucepan"],
      sections: brigadeiroSections(),
    });
    expect(pages).toHaveLength(1);
  });
});
