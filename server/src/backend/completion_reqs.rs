use super::Backend;
use tower_lsp::lsp_types::{CompletionItem, CompletionParams, CompletionResponse};
use super::document_handler::SymbolKind;
use tree_sitter::Point;

const KEYWORDS: &[&str] = &[
    "int", "void", "const", "if", "else", "while", 
    "break", "continue", "return"];
impl Backend {
    pub async fn completion_handler(&self, params: CompletionParams) -> Result<Option<CompletionResponse>,
    tower_lsp::jsonrpc::Error>
    {
        // Extract the text document position
        let mut position = params.text_document_position.position;
        if position.character > 0 {
            position.character -= 1;
        } 
        let uri = params.text_document_position.text_document.uri;
        
        // Get document content from the document store
        let doc = self.documents.get(&uri).expect("Document not found");
        let doc_handler = doc.lock().await;

        let last_token = doc_handler.get_token_at_position(position);
        if last_token.is_none() {
            return Ok(None);
        }
        let last_token = last_token.unwrap();
        let last_token_text = doc_handler.get_text_range(
                Point {
                    row: last_token.start.line as usize,
                    column: last_token.start.character as usize,
                },
                Point {
                    row: last_token.end.line as usize,
                    column: last_token.end.character as usize,
                },
            );

        let mut completions:Vec<CompletionItem> = Vec::new();
        for keyword in KEYWORDS {
            if keyword.starts_with(&last_token_text.to_lowercase()) {
                completions.push(CompletionItem {
                    label: keyword.to_string(),
                    kind: Some(tower_lsp::lsp_types::CompletionItemKind::KEYWORD),
                    ..Default::default()
                });
            }
        }
        let symbols = doc_handler.query_symbols(position);
        for symbol in symbols {
            if symbol.name.to_lowercase().starts_with(&last_token_text.to_lowercase()) {
                let kind = match symbol.kind {
                    SymbolKind::Function => tower_lsp::lsp_types::CompletionItemKind::FUNCTION,
                    SymbolKind::Variable => tower_lsp::lsp_types::CompletionItemKind::VARIABLE,
                    _ => tower_lsp::lsp_types::CompletionItemKind::TEXT,
                };
                completions.push(CompletionItem {
                    label: symbol.name,
                    kind: Some(kind),
                    ..Default::default()
                });
            };
        }
        Ok(
            Some(CompletionResponse::Array(
                completions
            ))
        )
    }
}