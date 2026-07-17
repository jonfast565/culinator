export interface UnitOption {
  value: string;
  label: string;
}

export interface UnitGroup {
  id: string;
  label: string;
  units: UnitOption[];
}

export const UNIT_GROUPS: UnitGroup[] = [
  {
    id: "mass",
    label: "Mass",
    units: [
      { value: "g", label: "grams (g)" },
      { value: "kg", label: "kilograms (kg)" },
      { value: "mg", label: "milligrams (mg)" },
      { value: "oz", label: "ounces (oz)" },
      { value: "lb", label: "pounds (lb)" },
    ],
  },
  {
    id: "volume",
    label: "Volume",
    units: [
      { value: "ml", label: "milliliters (ml)" },
      { value: "l", label: "liters (l)" },
      { value: "tsp", label: "teaspoons (tsp)" },
      { value: "tbsp", label: "tablespoons (tbsp)" },
      { value: "cup", label: "cups" },
      { value: "floz", label: "fluid ounces" },
      { value: "pt", label: "pints" },
      { value: "qt", label: "quarts" },
      { value: "gal", label: "gallons" },
      { value: "pinch", label: "pinches" },
      { value: "dash", label: "dashes" },
    ],
  },
  {
    id: "temperature",
    label: "Temperature",
    units: [
      { value: "c", label: "Celsius (°C)" },
      { value: "f", label: "Fahrenheit (°F)" },
    ],
  },
  {
    id: "time",
    label: "Time",
    units: [
      { value: "s", label: "seconds" },
      { value: "min", label: "minutes" },
      { value: "h", label: "hours" },
    ],
  },
  {
    id: "length",
    label: "Length",
    units: [
      { value: "mm", label: "millimeters" },
      { value: "cm", label: "centimeters" },
      { value: "m", label: "meters" },
      { value: "in", label: "inches" },
      { value: "ft", label: "feet" },
    ],
  },
];

export const ALL_UNITS: UnitOption[] = UNIT_GROUPS.flatMap((group) => group.units);

export function unitsInGroup(groupId: string): UnitOption[] {
  return UNIT_GROUPS.find((group) => group.id === groupId)?.units ?? ALL_UNITS;
}
