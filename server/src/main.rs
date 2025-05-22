use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService, Server, Client};
use tracing_subscriber;

struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult, tower_lsp::jsonrpc::Error> {
        self.client.log_message(MessageType::INFO, "Server initialized".to_string()).await;
        Ok(InitializeResult::default())
    }

    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();


    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
