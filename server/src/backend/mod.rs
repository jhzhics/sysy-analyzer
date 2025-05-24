use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, Client};

pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult, tower_lsp::jsonrpc::Error> {
        let mut capabilities = ServerCapabilities::default();

        capabilities.text_document_sync = Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                ..Default::default()
            }
        ));

        capabilities.hover_provider = Some(HoverProviderCapability::Simple(true));

        let initialize_result = InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        };

        self.client.log_message(MessageType::INFO, "Server initialized".to_string()).await;
        Ok(initialize_result)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) -> () {
        self.client.log_message(MessageType::LOG, format!("Opened src file: {}", params.text_document.uri)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::LOG, format!("Changed src file: {}", params.text_document.uri)).await;
        for change in params.content_changes {
            self.client.log_message(MessageType::LOG, format!("Change: {:?}", change)).await;
        }
        // You would typically process the changes here
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client.log_message(MessageType::LOG, format!("Closed src file: {}", params.text_document.uri)).await;
    }

    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        self.client.log_message(MessageType::LOG, "Server shutting down".to_string()).await;
        Ok(())
    }

    // Add other handlers as needed (e.g., hover, completion, etc.)
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>, tower_lsp::jsonrpc::Error> {
        self.client.log_message(MessageType::LOG, format!("Hover request at position: {:?}", params.text_document_position_params.position)).await;
        // Return a dummy hover for now
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("This is a hover! ✨".to_string())),
            range: None,
        }))
    }
}