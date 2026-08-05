/**
 * Measurement-driven packing. The provider is injected with heights this file
 * computes itself, so every assertion is exact: a page is full to the pixel the
 * measurements allow, and the block that starts the next card provably could not
 * have fit on the previous one.
 */
import { describe, expect, it } from "vitest";
import type {
  IngredientGroup,
  MethodSection,
  NarrativeStep,
  SectionMise,
} from "../recipe-editor/narrative";
import { INDEX_CARD_SPECS, type IndexCardFormat } from "./indexCardFormat";
import { INDEX_CARD_MARGIN_IN, type IndexCardMargin } from "./indexCardMargin";
import { indexCardBlockKey, type IndexCardMeasurements } from "./measureIndexCard";
import {
  cardContentBox,
  estimateIndexCardPageHeight,
  indexCardContinuationLabel,
  indexCardIngredientsSectionLabel,
  indexCardSectionLabel,
  paginateIndexCards,
  type IndexCardPage,
} from "./paginateIndexCards";

type CardFormat = Exclude<IndexCardFormat, "full">;

const ALL_CARD_FORMATS = Object.keys(INDEX_CARD_SPECS) as CardFormat[];
const ALL_MARGINS: IndexCardMargin[] = ["tight", "medium", "wide"];

const SECTION_LABELS = [
  "Ingredients",
  "Ingredients (cont.)",
  "Equipment",
  "Equipment (cont.)",
  "Method",
  "Method (cont.)",
];

/** Stand-in rendered heights — arbitrary but fixed, so packing is checkable. */
const FAKE = {
  titleHead: 46,
  titleCover: 30,
  continuationHead: 20,
  sectionLabel: 15,
  groupLabel: 12,
  rowLine: 14,
  rowChrome: 3,
  stepLine: 13,
  stepTimeExtra: 4,
  processHeading: 16,
  noteLine: 10,
  equipmentRow: 12,
  miseChrome: 18,
  miseLabel: 9,
  sectionGap: 8,
  firstSectionGap: 6,
  stepsGap: 2,
  pagerReserve: 13,
  charPx: 4.6,
};

function fakeLines(text: string, width: number): number {
  return Math.max(1, Math.ceil((text.length * FAKE.charPx) / Math.max(20, width)));
}

function fakeRowHeight(item: IngredientGroup["items"][number], width: number): number {
  const text = item.aside ? `${item.description} ${item.aside}` : item.description;
  return fakeLines(text, width - 52) * FAKE.rowLine + FAKE.rowChrome;
}

function fakeStepHeight(step: NarrativeStep, width: number): number {
  return fakeLines(step.text, width - 24) * FAKE.stepLine + (step.time ? FAKE.stepTimeExtra : 0);
}

function fakeEquipmentHeight(items: readonly string[], width: number): number {
  const columns = width < 280 ? 1 : 2;
  return Math.ceil(items.length / columns) * FAKE.equipmentRow;
}

function fakeMiseHeight(mise: SectionMise, width: number): number {
  let height = FAKE.miseChrome;
  if (mise.ingredients.length) {
    height += FAKE.miseLabel;
    for (const item of mise.ingredients) height += fakeRowHeight(item, width);
  }
  if (mise.equipment.length) {
    height += FAKE.miseLabel + fakeEquipmentHeight(mise.equipment, width);
  }
  return height;
}

function labelVariants(label: string | undefined): string[] {
  if (!label) return [];
  const continued = indexCardContinuationLabel(label);
  return continued && continued !== label ? [label, continued] : [label];
}

interface CardGeometry {
  contentWidth: number;
  contentHeight: number;
}

/** Physical content box, minus the 1px frame border the probe measures inside. */
function cardGeometry(format: CardFormat, margin: IndexCardMargin): CardGeometry {
  const spec = INDEX_CARD_SPECS[format];
  const marginIn = INDEX_CARD_MARGIN_IN[margin].card;
  return {
    contentWidth: (spec.widthIn - 2 * marginIn) * 96 - 2,
    contentHeight: (spec.heightIn - 2 * marginIn) * 96 - 2,
  };
}

interface CardContent {
  ingredientGroups: IngredientGroup[];
  equipment: string[];
  sections: MethodSection[];
}

/** Mirrors the key set `IndexCardProbe.vue` renders, with synthetic heights. */
function fakeMeasurements(content: CardContent, geometry: CardGeometry): IndexCardMeasurements {
  const width = geometry.contentWidth;
  const heights = new Map<string, number>();
  heights.set(indexCardBlockKey.titleHead, FAKE.titleHead);
  heights.set(indexCardBlockKey.titleCover, FAKE.titleCover);
  heights.set(indexCardBlockKey.continuationHead, FAKE.continuationHead);
  for (const label of SECTION_LABELS) {
    heights.set(indexCardBlockKey.sectionLabel(label), FAKE.sectionLabel);
  }
  for (const group of content.ingredientGroups) {
    for (const variant of labelVariants(group.label)) {
      heights.set(indexCardBlockKey.groupLabel(variant), FAKE.groupLabel);
    }
    for (const item of group.items) {
      heights.set(indexCardBlockKey.ingredientRow(item), fakeRowHeight(item, width));
    }
  }
  content.equipment.forEach((tool, index) => {
    const suffix = content.equipment.slice(index);
    heights.set(indexCardBlockKey.equipmentList(suffix), fakeEquipmentHeight(suffix, width));
    heights.set(indexCardBlockKey.equipmentList([tool]), fakeEquipmentHeight([tool], width));
  });
  for (const section of content.sections) {
    for (const variant of labelVariants(section.title)) {
      heights.set(indexCardBlockKey.processHeading(variant), FAKE.processHeading);
    }
    if (section.note) {
      heights.set(
        indexCardBlockKey.sectionNote(section.note),
        fakeLines(section.note, width) * FAKE.noteLine,
      );
    }
    if (section.mise.ingredients.length || section.mise.equipment.length) {
      heights.set(indexCardBlockKey.mise(section.mise), fakeMiseHeight(section.mise, width));
    }
    for (const step of section.steps) {
      heights.set(indexCardBlockKey.step(step), fakeStepHeight(step, width));
    }
  }
  return {
    heights,
    contentWidth: width,
    contentHeight: geometry.contentHeight,
    sectionGap: FAKE.sectionGap,
    firstSectionGap: FAKE.firstSectionGap,
    stepsGap: FAKE.stepsGap,
    pagerReserve: FAKE.pagerReserve,
  };
}

function metricsFor(format: CardFormat, margin: IndexCardMargin, measure: IndexCardMeasurements) {
  return cardContentBox(
    format,
    margin,
    { body: "md", header: "md", annotation: "md" },
    true,
    measure,
  );
}

function item(
  symbol: string,
  description: string,
  amount = "1",
  aside?: string,
): IngredientGroup["items"][number] {
  return { symbol, amount, description, aside };
}

function step(symbol: string, number: number, text: string, time?: string): NarrativeStep {
  return { symbol, number, text, time, tools: [] };
}

function emptyMise(): SectionMise {
  return { ingredients: [], equipment: [] };
}

function countIngredients(pages: IndexCardPage[]): number {
  return pages.reduce(
    (sum, page) => sum + page.ingredientGroups.reduce((n, group) => n + group.items.length, 0),
    0,
  );
}

function countSteps(pages: IndexCardPage[]): number {
  return pages.reduce(
    (sum, page) => sum + page.methodBlocks.reduce((n, block) => n + block.steps.length, 0),
    0,
  );
}

function applePieContent(): CardContent {
  return {
    ingredientGroups: [
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
    ],
    equipment: [
      "mixing bowl",
      "saucepan",
      "9-inch pie pan",
      "food processor",
      "rolling pin",
      "knife",
      "pastry brush",
      "oven",
    ],
    sections: [
      {
        process: "filling",
        title: "Filling",
        steps: [
          step(
            "cut",
            1,
            "Peel, core, and cut the golden delicious apples into even strips so they cook at the same rate.",
          ),
          step(
            "macerate",
            2,
            "Toss the apples with the sugars, cinnamon, nutmeg, cornstarch, and lemon juice, then let them rest until syrupy.",
            "30 min",
          ),
        ],
        mise: emptyMise(),
      },
      {
        process: "crust",
        title: "Crust",
        steps: [
          step(
            "mix",
            3,
            "Cut the cold butter into the flour, salt, and sugar until the mixture looks sandy with pea-sized pieces.",
          ),
          step(
            "roll",
            4,
            "Roll the dough out, line the pie pan, add the filling, top with the second crust, and brush with egg wash.",
          ),
          step(
            "bake",
            5,
            "Bake until the crust is deep golden and the filling bubbles thickly through the vents.",
            "1 hr",
          ),
        ],
        mise: emptyMise(),
      },
    ],
  };
}

function packApplePie(format: CardFormat, margin: IndexCardMargin) {
  const content = applePieContent();
  const measure = fakeMeasurements(content, cardGeometry(format, margin));
  const pages = paginateIndexCards({ format, margin, ...content, measure });
  return { content, measure, pages, m: metricsFor(format, margin, measure) };
}

/** Cost of moving the first block of `next` back onto `page`. */
function firstMethodBlockCost(
  page: IndexCardPage,
  next: IndexCardPage,
  measure: IndexCardMeasurements,
): number | null {
  const block = next.methodBlocks[0];
  if (!block || !page.methodBlocks.length) return null;
  const first = block.steps[0];
  const stepHeight = measure.heights.get(indexCardBlockKey.step(first));
  if (stepHeight == null) return null;
  // A “(cont.)” heading only exists because the section was split; the step
  // would simply rejoin the open block on the previous card.
  const heading =
    block.title && !block.title.endsWith("(cont.)")
      ? (measure.heights.get(indexCardBlockKey.processHeading(block.title)) ?? 0) + FAKE.stepsGap
      : 0;
  const mise = block.mise
    ? (measure.heights.get(indexCardBlockKey.mise(block.mise)) ?? 0) + FAKE.stepsGap
    : 0;
  return heading + mise + stepHeight + FAKE.stepsGap;
}

/** Cost of moving the first ingredient of `next` back onto `page`. */
function firstIngredientCost(
  page: IndexCardPage,
  next: IndexCardPage,
  measure: IndexCardMeasurements,
): number | null {
  if (!page.ingredientGroups.length || !next.ingredientGroups.length) return null;
  const group = next.ingredientGroups[0];
  const row = measure.heights.get(indexCardBlockKey.ingredientRow(group.items[0]));
  if (row == null) return null;
  const openGroup = page.ingredientGroups[page.ingredientGroups.length - 1];
  const continues =
    group.label === openGroup.label || group.label === indexCardContinuationLabel(openGroup.label);
  const label =
    continues || !group.label
      ? 0
      : (measure.heights.get(indexCardBlockKey.groupLabel(group.label)) ?? 0);
  return row + label;
}

function expectWithinBudget(
  pages: IndexCardPage[],
  m: ReturnType<typeof metricsFor>,
  measure: IndexCardMeasurements,
  context = "",
): void {
  for (const page of pages) {
    expect(
      estimateIndexCardPageHeight(page, m, { measure }),
      `${context} card ${page.index + 1}`,
    ).toBeLessThanOrEqual(m.height);
  }
}

function expectNoPrematureBreak(
  pages: IndexCardPage[],
  m: ReturnType<typeof metricsFor>,
  measure: IndexCardMeasurements,
  context = "",
): void {
  for (let index = 0; index < pages.length - 1; index += 1) {
    const page = pages[index];
    const used = estimateIndexCardPageHeight(page, m, { measure });
    const next = firstIngredientCost(page, pages[index + 1], measure);
    if (next != null) {
      expect(used + next, `${context} ingredients after card ${index + 1}`).toBeGreaterThan(
        m.height,
      );
    }
    const nextStep = firstMethodBlockCost(page, pages[index + 1], measure);
    if (nextStep != null) {
      expect(used + nextStep, `${context} method after card ${index + 1}`).toBeGreaterThan(
        m.height,
      );
    }
  }
}

describe("measured index-card packing", () => {
  it("fills a card to the last pixel the measurements allow", () => {
    const items = [
      item("a", "alpha"),
      item("b", "bravo"),
      item("c", "charlie"),
      item("d", "delta"),
    ];
    const heights = new Map<string, number>([
      [indexCardBlockKey.titleHead, 40],
      [indexCardBlockKey.continuationHead, 20],
      ...SECTION_LABELS.map((label) => [indexCardBlockKey.sectionLabel(label), 10] as const),
      ...items.map((entry) => [indexCardBlockKey.ingredientRow(entry), 20] as const),
    ]);
    // Exactly the title, the section chrome, and three of the four rows.
    const contentHeight = 40 + 6 + 10 + 3 * 20;
    const measure: IndexCardMeasurements = {
      heights,
      contentWidth: 300,
      contentHeight,
      sectionGap: 8,
      firstSectionGap: 6,
      stepsGap: 2,
      pagerReserve: 0,
    };
    const pages = paginateIndexCards({
      format: "6x4",
      margin: "medium",
      ingredientGroups: [{ items }],
      equipment: [],
      sections: [],
      showPager: false,
      measure,
    });
    const m = cardContentBox("6x4", "medium", {}, false, measure);

    expect(m.height).toBe(contentHeight);
    expect(pages).toHaveLength(2);
    expect(pages[0].ingredientGroups[0].items).toHaveLength(3);
    expect(estimateIndexCardPageHeight(pages[0], m, { measure })).toBe(contentHeight);
    expect(pages[1].ingredientGroups[0].items).toHaveLength(1);
    expect(countIngredients(pages)).toBe(4);
    expectWithinBudget(pages, m, measure);
  });

  it("packs steps to the measured budget with no safety multiplier", () => {
    const steps = Array.from({ length: 6 }, (_, index) =>
      step(`s${index}`, index + 1, `Step ${index + 1}`),
    );
    const heights = new Map<string, number>([
      [indexCardBlockKey.titleHead, 30],
      [indexCardBlockKey.continuationHead, 20],
      ...SECTION_LABELS.map((label) => [indexCardBlockKey.sectionLabel(label), 10] as const),
      ...steps.map((entry) => [indexCardBlockKey.step(entry), 20] as const),
    ]);
    // Title + first-section gap + label + four steps (three inter-step gaps).
    const contentHeight = 30 + 6 + 10 + 4 * 20 + 3 * 2;
    const measure: IndexCardMeasurements = {
      heights,
      contentWidth: 300,
      contentHeight,
      sectionGap: 8,
      firstSectionGap: 6,
      stepsGap: 2,
      pagerReserve: 0,
    };
    const pages = paginateIndexCards({
      format: "6x4",
      margin: "medium",
      ingredientGroups: [],
      equipment: [],
      sections: [{ process: "main", steps, mise: emptyMise() }],
      showPager: false,
      measure,
    });
    const m = cardContentBox("6x4", "medium", {}, false, measure);

    expect(pages[0].methodBlocks.flatMap((block) => block.steps)).toHaveLength(4);
    expect(estimateIndexCardPageHeight(pages[0], m, { measure })).toBe(contentHeight);
    expect(countSteps(pages)).toBe(6);
    expectWithinBudget(pages, m, measure);
    expectNoPrematureBreak(pages, m, measure);
  });

  it("keeps every Apple Pie block on a 6x4 card and never clips the method", () => {
    const { pages, measure, m, content } = packApplePie("6x4", "medium");

    expect(countIngredients(pages)).toBe(14);
    expect(countSteps(pages)).toBe(5);
    const placedEquipment = pages.flatMap((page) => page.equipment);
    expect(placedEquipment).toEqual(content.equipment);
    expectWithinBudget(pages, m, measure);
    expectNoPrematureBreak(pages, m, measure);
  });

  it("keeps equipment above the method and labels carried-over sections", () => {
    const { pages, measure, m } = packApplePie("3x5", "wide");

    const equipmentPage = pages.findIndex((page) => page.equipment.length > 0);
    const firstMethodPage = pages.findIndex((page) => page.methodBlocks.length > 0);
    expect(equipmentPage).toBeGreaterThanOrEqual(0);
    expect(firstMethodPage).toBeGreaterThanOrEqual(0);
    expect(equipmentPage).toBeLessThanOrEqual(firstMethodPage);
    const trailingEquipmentOnly = pages.some(
      (page, index) =>
        page.equipment.length > 0 &&
        page.methodBlocks.length === 0 &&
        pages.slice(0, index).some((prior) => prior.methodBlocks.length > 0),
    );
    expect(trailingEquipmentOnly).toBe(false);

    for (const page of pages) {
      if (page.ingredientGroups.length) {
        expect(indexCardIngredientsSectionLabel(page)).toBe(
          page.showTitle ? "Ingredients" : "Ingredients (cont.)",
        );
      }
      if (page.equipment.length && !page.showTitle && !page.ingredientGroups.length) {
        expect(indexCardSectionLabel("Equipment", page)).toBe("Equipment (cont.)");
      }
      if (page.methodBlocks.length && !page.showTitle && !page.ingredientGroups.length) {
        expect(indexCardSectionLabel("Method", page)).toBe("Method (cont.)");
      }
    }
    expectWithinBudget(pages, m, measure);
  });

  it("labels a method section that continues onto the next card", () => {
    const { pages } = packApplePie("2.5x4", "wide");
    const continued = pages
      .flatMap((page) => page.methodBlocks)
      .filter((block) => block.title?.endsWith("(cont.)"));
    for (const block of continued) {
      expect(block.title).toMatch(/ \(cont\.\)$/);
    }
    expect(countSteps(pages)).toBe(5);
  });

  it("budgets cover art on the title card", () => {
    const content = applePieContent();
    const geometry = cardGeometry("6x4", "medium");
    const measure = fakeMeasurements(content, geometry);
    const plain = paginateIndexCards({ format: "6x4", margin: "medium", ...content, measure });
    const withCover = paginateIndexCards({
      format: "6x4",
      margin: "medium",
      ...content,
      hasCover: true,
      measure,
    });
    const m = metricsFor("6x4", "medium", measure);

    const first = (pages: IndexCardPage[]): number =>
      pages[0].ingredientGroups.reduce((sum, group) => sum + group.items.length, 0);
    expect(first(withCover)).toBeLessThan(first(plain));
    expect(
      estimateIndexCardPageHeight(withCover[0], m, { measure, hasCover: true }),
    ).toBeLessThanOrEqual(m.height);
  });

  it("measures the colocated mise layout per section", () => {
    const content: CardContent = {
      ingredientGroups: [],
      equipment: [],
      sections: [
        {
          process: "cooking",
          title: "Cooking",
          note: "Start the sauce while the pasta water heats.",
          steps: [step("s1", 1, "Combine the condensed milk, chocolate, and butter.")],
          mise: {
            ingredients: [
              item("a", "condensed milk", "400 g"),
              item("b", "dark chocolate", "120 g"),
            ],
            equipment: ["saucepan"],
          },
        },
        {
          process: "finishing",
          title: "Finishing",
          steps: [step("s2", 2, "Cool the mixture and roll it in sprinkles.")],
          mise: { ingredients: [item("d", "sprinkles", "60 g")], equipment: [] },
        },
      ],
    };
    const measure = fakeMeasurements(content, cardGeometry("6x4", "medium"));
    const pages = paginateIndexCards({
      format: "6x4",
      margin: "medium",
      miseLayout: "colocated",
      ...content,
      measure,
    });
    const m = metricsFor("6x4", "medium", measure);

    expect(pages.every((page) => page.ingredientGroups.length === 0)).toBe(true);
    const mises = pages.flatMap((page) =>
      page.methodBlocks.filter((block) => block.mise).map((block) => block.mise),
    );
    expect(mises).toHaveLength(2);
    expect(countSteps(pages)).toBe(2);
    expectWithinBudget(pages, m, measure);
  });

  it("falls back to estimates for a block the probe never measured", () => {
    const content = applePieContent();
    const geometry = cardGeometry("6x4", "medium");
    const full = fakeMeasurements(content, geometry);
    const sparse: IndexCardMeasurements = {
      ...full,
      heights: new Map([[indexCardBlockKey.titleHead, FAKE.titleHead]]),
    };
    const pages = paginateIndexCards({
      format: "6x4",
      margin: "medium",
      ...content,
      measure: sparse,
    });
    expect(countIngredients(pages)).toBe(14);
    expect(countSteps(pages)).toBe(5);
    const m = metricsFor("6x4", "medium", sparse);
    expectWithinBudget(pages, m, sparse);
  });
});

describe("measured index-card packing (all presets)", () => {
  const presets = ALL_CARD_FORMATS.flatMap((format) =>
    ALL_MARGINS.map((margin) => ({ format, margin })),
  );

  it.each(presets)(
    "packs Apple Pie within the measured budget for $format / $margin",
    ({ format, margin }) => {
      const { pages, measure, m, content } = packApplePie(format, margin);
      const context = `${format}/${margin}`;

      expect(countIngredients(pages), context).toBe(14);
      expect(countSteps(pages), context).toBe(5);
      expect(pages.flatMap((page) => page.equipment).sort(), context).toEqual(
        [...content.equipment].sort(),
      );
      expect(
        pages.every((page) => page.total === pages.length),
        context,
      ).toBe(true);
      expectWithinBudget(pages, m, measure, context);
      expectNoPrematureBreak(pages, m, measure, context);
    },
  );
});
