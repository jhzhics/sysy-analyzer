use tower_lsp::lsp_types::{SemanticTokens, SemanticTokensParams, SemanticTokensResult};

use super::{document_handler::DocHandler, Backend};

impl Backend {
    pub async fn semantic_tokens_full_handler(&self, params: SemanticTokensParams) ->
    Result<Option<SemanticTokensResult>, tower_lsp::jsonrpc::Error> 
    {
        let uri = params.text_document.uri;
        let document_handler = self.documents.get(&uri).expect("Document not found");
        let doc_handler = document_handler.value();
        let semnatic_tokens = doc_handler.get_semantic_tokens();
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semnatic_tokens,
        })))
    }
    
}