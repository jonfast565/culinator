use culinator_application::{
    ApplicationError, BookService, ExportService, FormulaService, HaccpService, ImportService,
    KitchenService, NewRecipeBook, NutritionService, RecipeService, ScheduleService, SearchService,
    UnitService,
};
use culinator_export::{StaticRecipeBookExporter, StaticRecipeExporter};
use culinator_import::{
    JsonSettingsStore, OpenAiRecipeInterpreter, StructuredRecipeParser, TesseractCommandOcr,
};
use culinator_models::{
    CatalogRepository, DocumentParser, NutritionCatalog, RecipeImageAsset, RecipeImageData,
    RecipeValidator, UploadRecipeImageRequest,
};
use culinator_nutrition_fdc::{
    DEFAULT_FULL_DOWNLOAD_URL, SqliteNutritionCatalog, needs_full_catalog, seed_minimal_catalog,
};
use culinator_parser::CulinatorParser;
use culinator_scheduler::CriticalPathScheduler;
use culinator_secrets::resolve_secret_store;
use culinator_sqlite::SqliteCatalogRepository;
use culinator_validator::CulinatorValidator;
use std::{
    path::PathBuf,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};
use uuid::Uuid;

const FDC_RELEASE: &str = "2026-04";

/// Sample recipes used to seed a fresh catalog. These are Alton Brown recipes
/// converted into the Culinator DSL for demonstration; each carries a source
/// credit. See the `source_url` in each document for the original.
const SEED_RECIPES: &[&str] = &[
    include_str!("seed/baked_macaroni_and_cheese.cg"),
    include_str!("seed/easy_crepes.cg"),
    include_str!("seed/fully_loaded_guacamole.cg"),
];

#[derive(Clone)]
pub struct ServiceState {
    recipes: RecipeService,
    books: BookService,
    formulas: FormulaService,
    haccp: HaccpService,
    kitchen: KitchenService,
    nutrition: NutritionService,
    exports: ExportService,
    imports: ImportService,
    schedules: ScheduleService,
    search: SearchService,
    units: UnitService,
    images: Arc<dyn CatalogRepository>,
    fdc_path: PathBuf,
    nutrition_catalog: Arc<RwLock<Option<Arc<dyn NutritionCatalog>>>>,
}

impl ServiceState {
    pub fn sqlite(
        db_path: PathBuf,
        settings_path: PathBuf,
    ) -> Result<Self, culinator_application::ApplicationError> {
        let fdc_path = db_path.with_file_name("fdc.sqlite3");
        Self::sqlite_with_fdc(db_path, settings_path, fdc_path)
    }

    pub fn sqlite_with_fdc(
        db_path: PathBuf,
        settings_path: PathBuf,
        fdc_path: PathBuf,
    ) -> Result<Self, culinator_application::ApplicationError> {
        if !fdc_path.exists() {
            culinator_nutrition_fdc::seed_minimal_catalog(&fdc_path).map_err(|error| {
                culinator_application::ApplicationError::Internal(format!(
                    "nutrition starter catalog: {error}"
                ))
            })?;
        }
        let repository = Arc::new(SqliteCatalogRepository::new(db_path));
        repository.initialize()?;
        let nutrition_catalog = Arc::new(RwLock::new(open_nutrition_catalog(fdc_path.clone())));
        Ok(Self::with_dependencies(
            repository.clone(),
            Arc::new(CulinatorParser),
            Arc::new(CulinatorValidator),
            settings_path,
            nutrition_catalog,
            fdc_path,
        ))
    }

    pub fn with_dependencies(
        repository: Arc<dyn CatalogRepository>,
        parser: Arc<dyn DocumentParser>,
        validator: Arc<dyn RecipeValidator>,
        settings_path: PathBuf,
        nutrition_catalog: Arc<RwLock<Option<Arc<dyn NutritionCatalog>>>>,
        fdc_path: PathBuf,
    ) -> Self {
        let schedules = ScheduleService::new(parser.clone(), Arc::new(CriticalPathScheduler));
        let settings_dir = settings_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| settings_path.clone());
        let secrets = resolve_secret_store(&settings_dir);
        Self {
            recipes: RecipeService::new(repository.clone(), parser.clone(), validator),
            books: BookService::new(repository.clone()),
            formulas: FormulaService::new(repository.clone()),
            haccp: HaccpService::new(repository.clone()),
            kitchen: KitchenService::new(repository.clone(), repository.clone(), schedules.clone()),
            nutrition: NutritionService::new(
                repository.clone(),
                repository.clone(),
                parser.clone(),
                nutrition_catalog.clone(),
            ),
            exports: ExportService::new(
                repository.clone(),
                repository.clone(),
                parser.clone(),
                Arc::new(StaticRecipeExporter),
                Arc::new(StaticRecipeBookExporter),
            ),
            imports: ImportService::new(
                Arc::new(TesseractCommandOcr),
                Arc::new(OpenAiRecipeInterpreter::default()),
                Arc::new(StructuredRecipeParser),
                Arc::new(JsonSettingsStore::new(settings_path, secrets.clone())),
                secrets,
                parser.clone(),
                Arc::new(CulinatorValidator),
            ),
            schedules,
            search: SearchService::new(repository.clone()),
            units: UnitService::new(),
            images: repository,
            fdc_path,
            nutrition_catalog: nutrition_catalog.clone(),
        }
    }

    /// First-run orchestration: seed sample recipes when the catalog is empty and
    /// ensure the nutrition dictionary exists (starter immediately, full USDA import
    /// kicked off in the background when only the starter catalog is present).
    pub fn initialize(
        &self,
    ) -> Result<culinator_models::InitReport, culinator_application::ApplicationError> {
        self.ensure_nutrition_catalog()?;
        let recipe_count_before = self.recipes.list()?.len() as i64;
        let mut recipes_seeded = false;
        if recipe_count_before == 0 {
            self.seed_if_empty()?;
            recipes_seeded = true;
        }
        let recipe_count = self.recipes.list()?.len() as i64;
        Ok(culinator_models::InitReport {
            catalog_ready: true,
            recipes_seeded,
            nutrition_ready: self.nutrition.catalog_available(),
            nutrition_starter: needs_full_catalog(&self.fdc_path),
            recipe_count,
        })
    }

    pub fn init_status(
        &self,
    ) -> Result<culinator_models::InitReport, culinator_application::ApplicationError> {
        Ok(culinator_models::InitReport {
            catalog_ready: true,
            recipes_seeded: !self.recipes.list()?.is_empty(),
            nutrition_ready: self.nutrition.catalog_available(),
            nutrition_starter: needs_full_catalog(&self.fdc_path),
            recipe_count: self.recipes.list()?.len() as i64,
        })
    }

    fn ensure_nutrition_catalog(&self) -> Result<(), ApplicationError> {
        if !self.nutrition.catalog_available() {
            if !self.fdc_path.exists() {
                seed_minimal_catalog(&self.fdc_path).map_err(|error| {
                    ApplicationError::Internal(format!("nutrition starter catalog: {error}"))
                })?;
            }
            if let Some(catalog) = open_nutrition_catalog(self.fdc_path.clone()) {
                self.nutrition.set_catalog(Some(catalog));
            }
        }
        if needs_full_catalog(&self.fdc_path) {
            self.spawn_full_catalog_import();
        }
        Ok(())
    }

    fn spawn_full_catalog_import(&self) {
        static DOWNLOADING: AtomicBool = AtomicBool::new(false);
        if DOWNLOADING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }

        let fdc_path = self.fdc_path.clone();
        let catalog_slot = self.nutrition_catalog.clone();
        std::thread::spawn(move || {
            let _guard = DownloadGuard(&DOWNLOADING);
            eprintln!("Culinator: downloading USDA nutrition database…");
            match culinator_nutrition_fdc::download_and_build(
                &fdc_path,
                FDC_RELEASE,
                DEFAULT_FULL_DOWNLOAD_URL,
                true,
            ) {
                Ok(report) => {
                    eprintln!(
                        "Culinator: nutrition database ready (foods={}, nutrients={}, food_nutrients={})",
                        report.foods, report.nutrients, report.food_nutrients
                    );
                    if let Some(catalog) = open_nutrition_catalog(fdc_path) {
                        *catalog_slot
                            .write()
                            .expect("nutrition catalog lock poisoned") = Some(catalog);
                    }
                }
                Err(error) => {
                    eprintln!("Culinator: USDA nutrition download/import failed: {error}");
                }
            }
        });
    }

    pub fn list_recipe_images(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeImageAsset>, ApplicationError> {
        self.images.list_recipe_images(recipe_id)
    }
    pub fn get_recipe_image(
        &self,
        recipe_id: Uuid,
        handle: &str,
    ) -> Result<Option<RecipeImageData>, ApplicationError> {
        self.images.get_recipe_image(recipe_id, handle)
    }
    pub fn upload_recipe_image(
        &self,
        recipe_id: Uuid,
        input: UploadRecipeImageRequest,
    ) -> Result<RecipeImageAsset, ApplicationError> {
        self.images.upload_recipe_image(recipe_id, input)
    }
    pub fn delete_recipe_image(
        &self,
        recipe_id: Uuid,
        handle: &str,
    ) -> Result<bool, ApplicationError> {
        self.images.delete_recipe_image(recipe_id, handle)
    }

    /// Populate a brand-new catalog with a handful of sample recipes so the
    /// desktop app is never empty on first launch. Does nothing once the user
    /// has any recipes of their own. Individual failures are surfaced to the
    /// caller so a bad sample can't leave the catalog half-seeded silently.
    pub fn seed_if_empty(&self) -> Result<(), culinator_application::ApplicationError> {
        if !self.recipes.list()?.is_empty() {
            return Ok(());
        }
        let book = self.books.create(NewRecipeBook {
            title: "Sample Recipes".to_owned(),
            symbol: Some("sample_recipes".to_owned()),
            description: Some("Alton Brown classics to get you started".to_owned()),
        })?;
        for source in SEED_RECIPES {
            let created = self.recipes.create(Some(book.id))?;
            self.recipes.save(created.id, source)?;
        }
        Ok(())
    }

    pub fn recipes(&self) -> &RecipeService {
        &self.recipes
    }

    pub fn books(&self) -> &BookService {
        &self.books
    }

    pub fn formulas(&self) -> &FormulaService {
        &self.formulas
    }

    pub fn haccp(&self) -> &HaccpService {
        &self.haccp
    }

    pub fn kitchen(&self) -> &KitchenService {
        &self.kitchen
    }

    pub fn nutrition(&self) -> &NutritionService {
        &self.nutrition
    }

    pub fn imports(&self) -> &ImportService {
        &self.imports
    }

    pub fn schedules(&self) -> &ScheduleService {
        &self.schedules
    }

    pub fn exports(&self) -> &ExportService {
        &self.exports
    }

    pub fn search(&self) -> &SearchService {
        &self.search
    }

    pub fn units(&self) -> &UnitService {
        &self.units
    }
}

fn open_nutrition_catalog(path: PathBuf) -> Option<Arc<dyn NutritionCatalog>> {
    if !path.exists() {
        return None;
    }
    match SqliteNutritionCatalog::open(path) {
        Ok(catalog) => Some(Arc::new(catalog)),
        Err(error) => {
            eprintln!("Culinator nutrition database could not be opened: {error}");
            None
        }
    }
}

struct DownloadGuard<'a>(&'a AtomicBool);

impl Drop for DownloadGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(test)]
pub(crate) fn test_state() -> ServiceState {
    let base =
        std::env::temp_dir().join(format!("culinator-service-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&base).expect("create temp dir");
    ServiceState::sqlite(base.join("catalog.sqlite3"), base.join("settings.json"))
        .expect("initialize service state")
}

#[cfg(test)]
mod test;
