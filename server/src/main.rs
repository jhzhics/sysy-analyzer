use tower_lsp::{LspService, Server};

mod backend;
mod treap_list;

#[tokio::main]
async fn main() {

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();


    let (service, socket) = LspService::new(|client| backend::Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
