use culinograph_application::{BookService, ExportService, FormulaService, ImportService, RecipeService, ScheduleService};
use culinograph_models::{CatalogRepository, DocumentParser, RecipeValidator};
use culinograph_export::StaticRecipeExporter;
use culinograph_parser::CulinographParser;
use culinograph_sqlite::SqliteCatalogRepository;
use culinograph_validator::CulinographValidator;
use culinograph_scheduler::CriticalPathScheduler;
use std::{path::PathBuf, sync::Arc};
use culinograph_import::{JsonSettingsStore, OpenAiRecipeInterpreter, TesseractCommandOcr};

#[derive(Clone)]
pub struct ServiceState {
    recipes: RecipeService,
    books: BookService,
    formulas: FormulaService,
    exports: ExportService,
    imports: ImportService,
    schedules: ScheduleService,
}

impl ServiceState {
    pub fn sqlite(db_path: PathBuf, settings_path: PathBuf) -> Result<Self, culinograph_application::ApplicationError> {
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
        Self {
            recipes: RecipeService::new(repository.clone(), parser.clone(), validator),
            books: BookService::new(repository.clone()),
            formulas: FormulaService::new(repository.clone()),
            exports: ExportService::new(repository, parser.clone(), Arc::new(StaticRecipeExporter)),
            imports: ImportService::new(Arc::new(TesseractCommandOcr), Arc::new(OpenAiRecipeInterpreter::default()), Arc::new(JsonSettingsStore::new(settings_path)), parser.clone(), Arc::new(CulinographValidator)),
            schedules: ScheduleService::new(parser, Arc::new(CriticalPathScheduler)),
        }
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

    pub fn imports(&self) -> &ImportService { &self.imports }

    pub fn schedules(&self) -> &ScheduleService { &self.schedules }

    pub fn exports(&self) -> &ExportService {
        &self.exports
    }
}

#[cfg(test)]
mod test;
