use chumsky::Parser;
use linefeed::grammar::lexer::{self, Token};
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                semantic_tokens_provider: Some(
                    // TODO: This looks like crap... provide some kind of builder pattern instead
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions {
                                work_done_progress: Some(false),
                            },
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::NUMBER,   // 0
                                    SemanticTokenType::STRING,   // 1
                                    SemanticTokenType::REGEXP,   // 2
                                    SemanticTokenType::OPERATOR, // 3
                                    SemanticTokenType::KEYWORD,  // 4
                                    SemanticTokenType::VARIABLE, // 5
                                ],
                                token_modifiers: vec![],
                            },
                            range: None,
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        // Read file contents
        let src = std::fs::read_to_string(params.text_document.uri.path())
            .unwrap()
            .as_ref();

        // Break down the file contents into tokens using the lexer
        let tokens = match lexer::lexer().parse(src).into_output_errors() {
            (Some(tokens), e) if e.is_empty() => tokens,
            (_, e) => return Err(Error::method_not_found()), // TODO: Create better error here
        };

        let mut semantic_tokens = Vec::new();

        for token in tokens {
            let token_type = match token.0 {
                Token::Num(_) => 0,
                Token::Str(_) => 1,
                Token::Regex(_) => 2,
                Token::Op(_) => 3,
                Token::Ctrl(_) | Token::Bool(_) => 4,
                Token::Ident(_) => 5,
                Token::If
                | Token::Else
                | Token::Or
                | Token::And
                | Token::Not
                | Token::Xor
                | Token::Fn
                | Token::Return
                | Token::Unless
                | Token::While
                | Token::For
                | Token::In
                | Token::Break
                | Token::Continue
                | Token::Match
                | Token::RangeExclusive
                | Token::Null
                | Token::RangeInclusive => 4,
            };

            let token_result = SemanticToken {
                delta_line: 0,
                delta_start: 0,
                length: 0,
                token_type: token_type as u32,
                token_modifiers_bitset: 0 as u32,
            };

            semantic_tokens.push(token_result);
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
