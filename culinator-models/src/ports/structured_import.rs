use crate::{ApplicationError, ImportDraft, StructuredInput};

pub trait StructuredRecipeImporter: Send + Sync {
    fn import(&self, input: StructuredInput) -> Result<ImportDraft, ApplicationError>;
}
