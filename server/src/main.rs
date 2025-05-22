use tower_lsp::{LspService, Server};

mod backend;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();


    let (service, socket) = LspService::new(|client| backend::Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
