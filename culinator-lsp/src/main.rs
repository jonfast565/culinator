use tower_lsp::{Client, LanguageServer, LspService, Server, jsonrpc::Result, lsp_types::*};

struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Culinator language server initialized")
            .await;
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.publish(&params.text_document.uri, &params.text_document.text)
            .await;
    }
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            self.publish(&params.text_document.uri, &change.text).await;
        }
    }
    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(None)
    }
    async fn goto_definition(
        &self,
        _: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(None)
    }
    async fn rename(&self, _: RenameParams) -> Result<Option<WorkspaceEdit>> {
        Ok(None)
    }
    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("resource".into(), "Declare a typed resource".into()),
            CompletionItem::new_simple("operation".into(), "Declare a typed operation".into()),
            CompletionItem::new_simple("process".into(), "Declare a process".into()),
        ])))
    }
}

impl Backend {
    async fn publish(&self, uri: &Url, text: &str) {
        let diagnostics = match culinator_parser::parse_recipe(text) {
            Ok(recipe) => culinator_validator::validate(&recipe)
                .into_iter()
                .map(|d| Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(d.code.into())),
                    source: Some("culinator".into()),
                    message: d.message,
                    ..Default::default()
                })
                .collect(),
            Err(error) => vec![Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("culinator".into()),
                message: error.to_string(),
                ..Default::default()
            }],
        };
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
#[cfg(test)]
mod test;
