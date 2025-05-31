use super::Backend;
use tower_lsp::lsp_types::{CompletionItem, CompletionParams, CompletionResponse};
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

        let mut completions = vec![];
        for keyword in KEYWORDS {
            if keyword.starts_with(&last_token_text) {
                completions.push(keyword.to_string());
            }
        }
        Ok(
            Some(CompletionResponse::Array(
                completions.into_iter().map(|s| 
                CompletionItem {
                    label: s,
                    kind: Some(tower_lsp::lsp_types::CompletionItemKind::KEYWORD),
                    ..Default::default()
                }
                ).collect()
            ))
        )
    }
    
}