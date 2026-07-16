use culinograph_application::{
    BookService, ExportService, FormulaService, HaccpService, ImportService, KitchenService,
    NewRecipeBook, RecipeService, ScheduleService,
};
use culinograph_export::StaticRecipeExporter;
use culinograph_import::{JsonSettingsStore, OpenAiRecipeInterpreter, TesseractCommandOcr};
use culinograph_models::{CatalogRepository, DocumentParser, RecipeValidator};
use culinograph_parser::CulinographParser;
use culinograph_scheduler::CriticalPathScheduler;
use culinograph_sqlite::SqliteCatalogRepository;
use culinograph_validator::CulinographValidator;
use std::{path::PathBuf, sync::Arc};

/// Sample recipes used to seed a fresh catalog. These are Alton Brown recipes
/// converted into the Culinograph DSL for demonstration; each carries a source
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
    exports: ExportService,
    imports: ImportService,
    schedules: ScheduleService,
}

impl ServiceState {
    pub fn sqlite(
        db_path: PathBuf,
        settings_path: PathBuf,
    ) -> Result<Self, culinograph_application::ApplicationError> {
        let repository = Arc::new(SqliteCatalogRepository::new(db_path));
        repository.initialize()?;
        Ok(Self::with_dependencies(
            repository.clone(),
            Arc::new(CulinographParser),
            Arc::new(CulinographValidator),
            settings_path,
        ))
    }

    pub fn with_dependencies(
        repository: Arc<dyn CatalogRepository>,
        parser: Arc<dyn DocumentParser>,
        validator: Arc<dyn RecipeValidator>,
        settings_path: PathBuf,
    ) -> Self {
        let schedules = ScheduleService::new(parser.clone(), Arc::new(CriticalPathScheduler));
        Self {
            recipes: RecipeService::new(repository.clone(), parser.clone(), validator),
            books: BookService::new(repository.clone()),
            formulas: FormulaService::new(repository.clone()),
            haccp: HaccpService::new(repository.clone()),
            kitchen: KitchenService::new(repository.clone(), repository.clone(), schedules.clone()),
            exports: ExportService::new(repository, parser.clone(), Arc::new(StaticRecipeExporter)),
            imports: ImportService::new(
                Arc::new(TesseractCommandOcr),
                Arc::new(OpenAiRecipeInterpreter::default()),
                Arc::new(JsonSettingsStore::new(settings_path)),
                parser.clone(),
                Arc::new(CulinographValidator),
            ),
            schedules,
        }
    }

    /// Populate a brand-new catalog with a handful of sample recipes so the
    /// desktop app is never empty on first launch. Does nothing once the user
    /// has any recipes of their own. Individual failures are surfaced to the
    /// caller so a bad sample can't leave the catalog half-seeded silently.
    pub fn seed_if_empty(&self) -> Result<(), culinograph_application::ApplicationError> {
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

    pub fn imports(&self) -> &ImportService {
        &self.imports
    }

    pub fn schedules(&self) -> &ScheduleService {
        &self.schedules
    }

    pub fn exports(&self) -> &ExportService {
        &self.exports
    }
}

#[cfg(test)]
pub(crate) fn test_state() -> ServiceState {
    let base =
        std::env::temp_dir().join(format!("culinograph-service-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&base).expect("create temp dir");
    ServiceState::sqlite(base.join("catalog.sqlite3"), base.join("settings.json"))
        .expect("initialize service state")
}

#[cfg(test)]
mod test;
