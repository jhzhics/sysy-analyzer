use std::ops::{DerefMut};

use tokio::sync::Mutex;
use tower_lsp::lsp_types::{DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams};
use super::Backend;
use super::document_handler::DocHandler;

impl Backend {
    pub async fn did_open_handler(&self, params: DidOpenTextDocumentParams) {
        let text = &params.text_document.text;
        let mut parser = self.parser.lock().await;
        let parser_ref = parser.deref_mut();
        let doc_handler = DocHandler::new(text, parser_ref);
        self.documents.insert(params.text_document.uri.clone(), 
            Mutex::new(doc_handler));
    }

    pub async fn did_change_handler(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let handler = self.documents.get_mut(&uri).expect("Document not found");
        let mut handler = handler.lock().await;
        let mut parser = self.parser.lock().await;
        for change in params.content_changes {
            handler.incremental_update(&change, parser.deref_mut()).await;
        }
    }

    pub async fn did_close_handler(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
    }
}