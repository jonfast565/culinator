use async_trait::async_trait;
use culinograph_models::{ApplicationError, ImportSettings, RecipeImage, RecipeImageInterpreter};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct OpenAiRecipeInterpreter { client: reqwest::Client }
impl Default for OpenAiRecipeInterpreter { fn default()->Self { Self { client:reqwest::Client::new() } } }

#[async_trait]
impl RecipeImageInterpreter for OpenAiRecipeInterpreter {
    async fn interpret(&self, images:&[RecipeImage], extracted_text:Option<&str>, target_language:Option<&str>, settings:&ImportSettings)->Result<(String,String,Vec<String>),ApplicationError>{
        if settings.openai_api_key.trim().is_empty(){return Err(ApplicationError::InvalidInput("OpenAI API key is not configured".into()));}
        let language=target_language.unwrap_or("the source recipe language");
        let mut content=vec![json!({"type":"input_text","text":format!("You are translating a photographed recipe into Culinograph DSL version 0.3. Preserve exact quantities, units, temperatures, timing, sequencing, dependencies, yields, and notes. Use readable declarations such as `ingredient flour measured by mass`, `process mixing`, and `operation combine does mix`. Create syntactically valid DSL with a single recipe. Write human-facing title and notes in {language}. Never invent missing quantities; add a warning instead. Return ONLY JSON with keys title, sourceText, extractedText, warnings.\n\nCulinograph skeleton:\nculinograph 0.3;\nrecipe recipe_symbol {{ title \"Title\"; ingredient ...; process ...; yield ...; serving ...; }}\n\nLocal OCR text:\n{}",extracted_text.unwrap_or("(none; inspect the images directly)"))})];
        if extracted_text.is_none(){ for image in images { content.push(json!({"type":"input_image","image_url":format!("data:{};base64,{}",image.media_type,image.data_base64),"detail":"high"})); } }
        let body=json!({"model":settings.openai_model,"input":[{"role":"user","content":content}],"text":{"format":{"type":"json_schema","name":"culinograph_recipe_import","strict":true,"schema":{"type":"object","additionalProperties":false,"properties":{"title":{"type":"string"},"sourceText":{"type":"string"},"extractedText":{"type":"string"},"warnings":{"type":"array","items":{"type":"string"}}},"required":["title","sourceText","extractedText","warnings"]}}}});
        let response=self.client.post("https://api.openai.com/v1/responses").bearer_auth(&settings.openai_api_key).json(&body).send().await.map_err(|e|ApplicationError::Internal(format!("OpenAI request failed: {e}")))?;
        let status=response.status(); let value:Value=response.json().await.map_err(|e|ApplicationError::Internal(format!("invalid OpenAI response: {e}")))?;
        if !status.is_success(){return Err(ApplicationError::Internal(format!("OpenAI API returned {status}: {}",value.get("error").and_then(|v|v.get("message")).and_then(Value::as_str).unwrap_or("unknown error"))));}
        let text=value.get("output_text").and_then(Value::as_str).map(str::to_owned).or_else(|| value.get("output").and_then(Value::as_array).and_then(|a|a.iter().flat_map(|o|o.get("content").and_then(Value::as_array).into_iter().flatten()).find_map(|c|c.get("text").and_then(Value::as_str).map(str::to_owned)))).ok_or_else(||ApplicationError::Internal("OpenAI response did not contain output text".into()))?;
        let parsed:Value=serde_json::from_str(&text).map_err(|e|ApplicationError::Internal(format!("OpenAI output was not valid JSON: {e}")))?;
        let title=parsed.get("title").and_then(Value::as_str).unwrap_or("Imported Recipe").to_owned();
        let source=parsed.get("sourceText").and_then(Value::as_str).ok_or_else(||ApplicationError::Internal("OpenAI output omitted sourceText".into()))?.to_owned();
        let extracted=parsed.get("extractedText").and_then(Value::as_str).unwrap_or(extracted_text.unwrap_or("")).to_owned();
        let mut warnings: Vec<String>=parsed.get("warnings").and_then(Value::as_array).map(|v|v.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
        if extracted.is_empty() { warnings.push("No OCR transcript was returned".to_owned()); }
        Ok((title, source, warnings))
    }
}
#[cfg(test)] mod test;
