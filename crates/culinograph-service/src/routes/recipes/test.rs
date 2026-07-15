use super::*;
#[tokio::test] async fn validation_reports_invalid_source() { let Json(result)=validate(Json(ValidateRequest{source_text:"not a recipe".into()})).await; assert!(!result.valid); assert!(!result.diagnostics.is_empty()); }
