use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, Client};

pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult, tower_lsp::jsonrpc::Error> {

        let capabilities = ServerCapabilities::default();

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

    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        self.client.log_message(MessageType::INFO, "Server shutting down".to_string()).await;
        Ok(())
    }
}