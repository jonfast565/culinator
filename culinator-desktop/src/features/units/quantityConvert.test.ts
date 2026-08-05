import { describe, expect, it } from "vitest";
import {
  detectAuthoredTemperatureScale,
  detectAuthoredUnitSystem,
  dominantIngredientDimension,
  temperatureScaleOf,
  unitSystemOf,
} from "./quantityConvert";

describe("unitSystemOf", () => {
  it("classifies mass and volume units", () => {
    expect(unitSystemOf("g")).toBe("metric");
    expect(unitSystemOf("ml")).toBe("metric");
    expect(unitSystemOf("cup")).toBe("us_customary");
    expect(unitSystemOf("oz")).toBe("us_customary");
    expect(unitSystemOf("count")).toBeNull();
  });
});

describe("dominantIngredientDimension", () => {
  it("prefers volume when cups and spoons outnumber weights", () => {
    expect(dominantIngredientDimension(["1 cup", "2 tbsp", "1 tsp", "100 g", "1 count"])).toBe(
      "volume",
    );
  });

  it("prefers mass when weights dominate", () => {
    expect(dominantIngredientDimension(["500 g", "250 g", "10 g", "1 tsp"])).toBe("mass");
  });

  it("returns null for count-only or ties", () => {
    expect(dominantIngredientDimension(["2 count", "1 clove"])).toBeNull();
    expect(dominantIngredientDimension(["1 cup", "100 g"])).toBeNull();
  });
});

describe("detectAuthoredUnitSystem", () => {
  it("keeps US volume recipes in US customary so cups stay cups", () => {
    expect(detectAuthoredUnitSystem(["2 cups", "1 tbsp", "1 tsp", "1 count", "2 cloves"])).toBe(
      "us_customary",
    );
  });

  it("keeps metric mass recipes in metric so grams stay grams", () => {
    expect(detectAuthoredUnitSystem(["500 g", "250 g", "10 g", "5 ml"])).toBe("metric");
  });

  it("follows the dominant dimension when systems mix", () => {
    // Mostly US volume, with one metric mass garnish — stay US.
    expect(detectAuthoredUnitSystem(["1 cup", "2 tbsp", "1 tsp", "5 g"])).toBe("us_customary");
    // Mostly metric mass, with one US volume — stay metric.
    expect(detectAuthoredUnitSystem(["500 g", "250 g", "50 g", "1 tsp"])).toBe("metric");
  });

  it("leaves preference alone for count-only recipes", () => {
    expect(detectAuthoredUnitSystem(["2 count", "1 clove", "3 pieces"])).toBeNull();
  });

  it("leaves preference alone when systems tie within the dominant dimension", () => {
    expect(detectAuthoredUnitSystem(["100 g", "4 oz"])).toBeNull();
  });
});

describe("temperatureScaleOf", () => {
  it("classifies temperature units", () => {
    expect(temperatureScaleOf("c")).toBe("celsius");
    expect(temperatureScaleOf("celsius")).toBe("celsius");
    expect(temperatureScaleOf("f")).toBe("fahrenheit");
    expect(temperatureScaleOf("fahrenheit")).toBe("fahrenheit");
    expect(temperatureScaleOf("g")).toBeNull();
  });
});

describe("detectAuthoredTemperatureScale", () => {
  it("prefers Fahrenheit when oven temps are mostly °F", () => {
    expect(
      detectAuthoredTemperatureScale(["350 f", "375 fahrenheit", "165 f"]),
    ).toBe("fahrenheit");
  });

  it("prefers Celsius when oven and doneness temps are mostly °C", () => {
    expect(
      detectAuthoredTemperatureScale(["180 celsius", "200 c", "68 celsius"]),
    ).toBe("celsius");
  });

  it("returns null when no temperatures are present", () => {
    expect(detectAuthoredTemperatureScale([])).toBeNull();
  });

  it("returns null when Celsius and Fahrenheit tie", () => {
    expect(detectAuthoredTemperatureScale(["350 f", "180 celsius"])).toBeNull();
  });
});
