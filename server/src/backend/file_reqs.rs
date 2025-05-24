use std::ops::{DerefMut};

use tower_lsp::lsp_types::{DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams};
use super::Backend;
use super::document_handler::DocHandler;

impl Backend {
    pub async fn did_open_handler(&self, params: DidOpenTextDocumentParams) {
        let text = &params.text_document.text;
        let mut parser = self.parser.lock().await;
        let parser_ref = parser.deref_mut();
        let doc_handler = DocHandler::new(text, parser_ref);
        self.documents.insert(params.text_document.uri.clone(), doc_handler);
    }

    pub async fn did_change_handler(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let handler = self.documents.get_mut(&uri).expect("Document not found").deref_mut();
        let mut parser = self.parser.lock().await.deref_mut();
        
    }

    pub async fn did_close_handler(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
    }
}