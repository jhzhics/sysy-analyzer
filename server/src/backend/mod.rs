use tokio::sync::Mutex;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, Client};
use dashmap::DashMap;
use std::sync::Arc;

mod document_handler;
mod file_reqs;
mod definition_req;

#[allow(dead_code)]
const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::NUMBER,
    SemanticTokenType::COMMENT,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::TYPE,
];

pub struct Backend {
    pub client: Client,
    documents: Arc<DashMap<Url, Mutex<document_handler::DocHandler>>>,
    parser: Arc<Mutex<tree_sitter::Parser>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_sysy_parser::LANGUAGE.into()).expect("Error loading Sysy grammar");

        Backend {
            client,
            documents: Arc::new(DashMap::new()),
            parser: Arc::new(Mutex::new(parser)),
        }
    }
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
        capabilities.definition_provider = Some(OneOf::Left(true));

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
        self.did_open_handler(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::LOG, format!("Changed src file: {}", params.text_document.uri)).await;
        for change in &params.content_changes {
            self.client.log_message(MessageType::LOG, format!("Change: {:?}", change)).await;
        }
        self.did_change_handler(params).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client.log_message(MessageType::LOG, format!("Closed src file: {}", params.text_document.uri)).await;
        self.did_close_handler(params).await;
    }

    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        self.client.log_message(MessageType::LOG, "Server shutting down".to_string()).await;
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>, tower_lsp::jsonrpc::Error> {
        self.client.log_message(MessageType::LOG, format!("Hover request at position: {:?}", params.text_document_position_params.position)).await;
        self.hover_handler(params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>,
        tower_lsp::jsonrpc::Error
    > {
        self.client.log_message(MessageType::LOG, format!("Goto declaration request at position: {:?}", params.text_document_position_params.position)).await;
        self.goto_definition_handler(params).await
    }
}