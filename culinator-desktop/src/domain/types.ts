export interface RecipeBookSummary {
  id: string;
  symbol: string;
  title: string;
  description?: string | null;
  protocolVersion: string;
  recipeCount: number;
  updatedAt: string;
}

export interface RecipeSummary {
  id: string;
  bookId?: string | null;
  symbol: string;
  title: string;
  protocolVersion: string;
  updatedAt: string;
}

export interface RecipeDocument extends RecipeSummary {
  sourceText: string;
}

export interface Diagnostic {
  severity: "error" | "warning" | "info" | "information";
  message: string;
  start?: number;
  end?: number;
}

export interface RecipeOutline {
  title: string;
  symbol: string;
  protocolVersion: string;
  typeCount: number;
  resourceCount: number;
  processCount: number;
  operationCount: number;
  servingCount: number;
}

export interface ValidationResult {
  valid: boolean;
  diagnostics: Diagnostic[];
  outline?: RecipeOutline;
}

export type FormulaBasis = "reference_percent" | "percent_of_total" | "absolute_mass";

export interface FormulaIngredient {
  id: string;
  symbol: string;
  name: string;
  stage: string;
  basis: FormulaBasis;
  percentage?: number | null;
  mass_grams?: number | null;
  is_reference: boolean;
  is_flour: boolean;
  water_fraction: number;
  scalable: boolean;
  properties: Record<string, unknown>;
}

export interface Formula {
  id: string;
  recipe_id?: string | null;
  symbol: string;
  name: string;
  basis: FormulaBasis;
  ingredients: FormulaIngredient[];
  properties: Record<string, unknown>;
}

export interface FormulaLineResult {
  ingredient_id: string;
  symbol: string;
  name: string;
  stage: string;
  percentage?: number | null;
  mass_grams: number;
  is_reference: boolean;
  is_flour: boolean;
  total_percentage: number;
}

export interface FormulaResult {
  target_mass_grams: number;
  reference_mass_grams: number;
  total_flour_grams: number;
  total_mass_grams: number;
  hydration_percent: number;
  prefermented_flour_percent: number;
  lines: FormulaLineResult[];
}

export type PercentageView = "reference" | "total";
export interface PercentageConversion {
  view: PercentageView;
  reference_mass_grams: number;
  total_mass_grams: number;
  lines: FormulaLineResult[];
}

export interface NutritionFacts {
  servingsPerContainer: number;
  servingSize: string;
  servingSizeGrams?: number | null;
  calories: number;
  totalFatGrams: number;
  saturatedFatGrams: number;
  transFatGrams: number;
  cholesterolMilligrams: number;
  sodiumMilligrams: number;
  totalCarbohydrateGrams: number;
  dietaryFiberGrams: number;
  totalSugarsGrams: number;
  addedSugarsGrams: number;
  proteinGrams: number;
  vitaminDMicrograms?: number | null;
  calciumMilligrams?: number | null;
  ironMilligrams?: number | null;
  potassiumMilligrams?: number | null;
}
export type RecipeExportFormat =
  "web" | "markdown" | "plain_text" | "ingredient_csv" | "json" | "print_html" | "epub";
export interface RecipeExportOptions {
  siteTitle?: string | null;
  author?: string | null;
  description?: string | null;
  includeSource: boolean;
  formats: RecipeExportFormat[];
  nutrition: NutritionFacts;
}
export interface RecipeExportResponse {
  fileName: string;
  mediaType: string;
  archiveBase64: string;
  files: string[];
}

export interface PublicImportSettings {
  apiKeyConfigured: boolean;
  openaiModel: string;
  useLocalOcr: boolean;
  tesseractCommand: string;
}
export interface RecipeImageInput {
  fileName: string;
  mediaType: string;
  dataBase64: string;
}
export interface RecipeImportResult {
  title: string;
  sourceText: string;
  extractedText: string;
  warnings: string[];
  validation: ValidationResult;
}

export interface ScheduledOperation {
  symbol: string;
  process: string;
  action: string;
  startSeconds: number;
  endSeconds: number;
  durationSeconds: number;
  labor?: string | null;
  dependencies: string[];
  resources: string[];
}
export interface RecipeSchedule {
  operations: ScheduledOperation[];
  makespanSeconds: number;
  criticalPath: string[];
}

export type HaccpPlanStatus = "draft" | "active" | "archived";
export type HazardType = "biological" | "chemical" | "physical";
export type HazardSeverity = "low" | "medium" | "high" | "critical";
export type HazardLikelihood = "unlikely" | "possible" | "likely" | "certain";

export interface HaccpPlanSummary {
  id: string;
  recipeId: string;
  title: string;
  description?: string | null;
  status: HaccpPlanStatus;
  hazardCount: number;
  ccpCount: number;
  updatedAt: string;
}

export interface HaccpHazard {
  id: string;
  position: number;
  hazardType: HazardType;
  description: string;
  severity: HazardSeverity;
  likelihood: HazardLikelihood;
  preventiveMeasures?: string | null;
  isCcp: boolean;
}

export interface HaccpCcp {
  id: string;
  hazardId?: string | null;
  position: number;
  name: string;
  operationSymbol?: string | null;
  criticalLimit: string;
  monitoringProcedure: string;
  monitoringFrequency?: string | null;
  correctiveAction: string;
  verificationProcedure?: string | null;
  responsibleParty?: string | null;
}

export interface HaccpMonitoringRecord {
  id: string;
  ccpId: string;
  recordedAt: string;
  measuredValue: string;
  withinLimit: boolean;
  correctiveActionTaken?: string | null;
  recordedBy?: string | null;
  notes?: string | null;
}

export interface HaccpPlanDocument {
  id: string;
  recipeId: string;
  title: string;
  description?: string | null;
  status: HaccpPlanStatus;
  hazards: HaccpHazard[];
  ccps: HaccpCcp[];
  monitoringRecords: HaccpMonitoringRecord[];
  updatedAt: string;
}

export type RecipeTryStatus = "active" | "paused" | "completed" | "abandoned";
export type TryOperationStatus = "pending" | "active" | "completed" | "skipped";

export interface RecipeTrySummary {
  id: string;
  recipeId: string;
  title?: string | null;
  status: RecipeTryStatus;
  startedAt?: string | null;
  completedAt?: string | null;
  operationCount: number;
  observationCount: number;
}

export interface TryOperation {
  operationId: string;
  operationSymbol: string;
  status: TryOperationStatus;
  scheduledStart?: string | null;
  scheduledEnd?: string | null;
  actualStart?: string | null;
  actualEnd?: string | null;
  durationSeconds: number;
  notes?: string | null;
}

export interface TryObservation {
  id: string;
  operationId?: string | null;
  operationSymbol?: string | null;
  observedAt: string;
  propertyPath: string;
  value: string | number | boolean | null;
  unit?: string | null;
  notes?: string | null;
}

export interface RecipeTryDocument {
  id: string;
  recipeId: string;
  recipeRevisionId?: string | null;
  title?: string | null;
  status: RecipeTryStatus;
  scaleFactor: number;
  startedAt?: string | null;
  completedAt?: string | null;
  notes?: string | null;
  findings?: string | null;
  operations: TryOperation[];
  observations: TryObservation[];
}

export interface NutritionSearchResult {
  fdcId: number;
  description: string;
  dataType: string;
  brandOwner?: string | null;
  servingSize?: number | null;
  servingSizeUnit?: string | null;
}

export interface ResourceNutritionLink {
  recipeId: string;
  resourceSymbol: string;
  fdcId: number;
  foodDescription: string;
  linkedAt: string;
}

export interface RecipeIngredientNutrition {
  resourceSymbol: string;
  resourceName?: string | null;
  massGrams?: number | null;
  fdcId?: number | null;
  foodDescription?: string | null;
  linked: boolean;
}

export interface RecipeNutritionResult {
  facts: NutritionFacts;
  totalMassGrams: number;
  linkedIngredientCount: number;
  totalIngredientCount: number;
  ingredients: RecipeIngredientNutrition[];
  warnings: string[];
}

export interface NutritionCatalogStatus {
  catalogAvailable: boolean;
}
