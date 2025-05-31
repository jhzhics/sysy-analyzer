use tower_lsp::lsp_types::{Hover, HoverParams, HoverContents, Position};
use tree_sitter::Point;

use super::Backend;
impl Backend {
    pub async fn hover_handler(&self, params: HoverParams) -> Result<Option<Hover>, tower_lsp::jsonrpc::Error> {
        let doc_handler = self.documents.get(&params.text_document_position_params.text_document.uri)
            .ok_or_else(|| tower_lsp::jsonrpc::Error::invalid_params("Document not found"))?;
        let doc = doc_handler.lock().await;
        let pos = Position {
            line: params.text_document_position_params.position.line,
            character: params.text_document_position_params.position.character,
        };
        let definition = doc.find_definition(pos).ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Definition not found at the given position")
        })?;
        let mut definition_text = doc.get_text_range(
            Point {
                row: definition.start.line as usize,
                column: definition.start.character as usize,
            },
            Point {
                row: definition.end.line as usize,
                column: definition.end.character as usize,
            },
        );

        if definition_text.len() > 200
        {
            definition_text.truncate(200);
            definition_text.push_str("...");
        }

        Ok(
            Some(Hover {
                contents: HoverContents::Scalar(
                    tower_lsp::lsp_types::MarkedString::LanguageString(
                        tower_lsp::lsp_types::LanguageString {
                            language: "sysy".to_string(), // Using 'c' language for SysY (similar to C)
                            value: definition_text,
                        }
                    )
                ),
                range: None,
            })
        )
    }
}