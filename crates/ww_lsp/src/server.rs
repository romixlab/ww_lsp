//! The language server: LSP over stdin and stdout with `lsp-server` (rust-analyzer's own, synchronous), full
//! document sync, inlay hints, hover and semantic tokens. Each open document is analysed on every change; a text that
//! doesn't parse (in the middle of typing) keeps the hints and tokens of the last one that did, at their old
//! positions.

use crate::Error;
use lsp_server::{Connection, ExtractError, Message, Notification, Request, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
};
use lsp_types::request::{
    HoverRequest, InlayHintRequest, Request as _, SemanticTokensFullRequest,
    SemanticTokensRangeRequest,
};
use lsp_types::{
    Hover, HoverContents, HoverProviderCapability, InitializeResult, InlayHint, InlayHintLabel,
    InlayHintTooltip, MarkupContent, MarkupKind, OneOf, Position, Range, SemanticToken,
    SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensRangeResult, SemanticTokensResult,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};
use std::collections::HashMap;
use std::process::ExitCode;
use ww_lsp_core::{Found, LineIndex, Token, TokenKind};

/// An open document: the analysis and the tokens of its last text that parsed, and the index of that text.
struct Doc {
    index: LineIndex,
    found: Vec<Found>,
    tokens: Vec<Token>,
}

impl Doc {
    fn range(&self, r: &std::ops::Range<usize>) -> Range {
        Range::new(self.pos(r.start), self.pos(r.end))
    }

    fn pos(&self, offset: usize) -> Position {
        let (line, col) = self.index.position(offset);
        Position::new(line, col)
    }

    fn offset(&self, p: Position) -> usize {
        self.index.offset(p.line, p.character)
    }
}

pub(crate) fn run() -> Result<ExitCode, Error> {
    let (connection, io_threads) = Connection::stdio();
    serve(&connection)?;
    drop(connection);
    io_threads.join()?;
    Ok(ExitCode::SUCCESS)
}

/// Initialise, then answer until `shutdown`.
pub(crate) fn serve(connection: &Connection) -> Result<(), Error> {
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        inlay_hint_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: legend(),
                range: Some(true),
                full: Some(SemanticTokensFullOptions::Bool(true)),
                work_done_progress_options: lsp_types::WorkDoneProgressOptions::default(),
            },
        )),
        ..ServerCapabilities::default()
    };
    let result = InitializeResult {
        capabilities,
        server_info: Some(ServerInfo {
            name: "ww_lsp".into(),
            version: Some(crate::VERSION.into()),
        }),
    };
    let (id, _params) = connection.initialize_start()?;
    connection.initialize_finish(id, serde_json::to_value(result)?)?;

    let mut docs: HashMap<String, Doc> = HashMap::new();
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let response = answer(&docs, req);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(n) => notified(&mut docs, n),
            Message::Response(_) => {}
        }
    }
    Ok(())
}

fn notified(docs: &mut HashMap<String, Doc>, n: Notification) {
    let (uri, text) = match n.method.as_str() {
        DidOpenTextDocument::METHOD => {
            match n.extract::<lsp_types::DidOpenTextDocumentParams>(DidOpenTextDocument::METHOD) {
                Ok(p) => (p.text_document.uri.to_string(), p.text_document.text),
                Err(_) => return,
            }
        }
        DidChangeTextDocument::METHOD => {
            match n.extract::<lsp_types::DidChangeTextDocumentParams>(DidChangeTextDocument::METHOD)
            {
                // full sync: the last change is the whole text
                Ok(mut p) => match p.content_changes.pop() {
                    Some(c) => (p.text_document.uri.to_string(), c.text),
                    None => return,
                },
                Err(_) => return,
            }
        }
        DidCloseTextDocument::METHOD => {
            if let Ok(p) =
                n.extract::<lsp_types::DidCloseTextDocumentParams>(DidCloseTextDocument::METHOD)
            {
                docs.remove(p.text_document.uri.as_str());
            }
            return;
        }
        _ => return,
    };
    if let (Ok(found), Ok(tokens)) = (ww_lsp_core::analyze(&text), ww_lsp_core::tokens(&text)) {
        docs.insert(
            uri,
            Doc {
                index: LineIndex::new(&text),
                found,
                tokens,
            },
        );
    }
}

fn answer(docs: &HashMap<String, Doc>, req: Request) -> Response {
    let id = req.id.clone();
    let result = match req.method.as_str() {
        InlayHintRequest::METHOD => params::<InlayHintRequest>(req).map(|p| {
            let hints = docs
                .get(p.text_document.uri.as_str())
                .map(|doc| inlay_hints(doc, p.range));
            serde_json::to_value(hints.unwrap_or_default())
        }),
        HoverRequest::METHOD => params::<HoverRequest>(req).map(|p| {
            let at = p.text_document_position_params;
            let hover = docs
                .get(at.text_document.uri.as_str())
                .and_then(|doc| hover(doc, at.position));
            serde_json::to_value(hover)
        }),
        SemanticTokensFullRequest::METHOD => params::<SemanticTokensFullRequest>(req).map(|p| {
            let data = docs
                .get(p.text_document.uri.as_str())
                .map(|doc| semantic_tokens(doc, None));
            serde_json::to_value(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: data.unwrap_or_default(),
            }))
        }),
        SemanticTokensRangeRequest::METHOD => params::<SemanticTokensRangeRequest>(req).map(|p| {
            let data = docs
                .get(p.text_document.uri.as_str())
                .map(|doc| semantic_tokens(doc, Some(p.range)));
            serde_json::to_value(SemanticTokensRangeResult::Tokens(SemanticTokens {
                result_id: None,
                data: data.unwrap_or_default(),
            }))
        }),
        _ => {
            return Response::new_err(
                id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                format!("ww_lsp doesn't answer {}", req.method),
            );
        }
    };
    match result {
        Ok(Ok(value)) => Response::new_ok(id, value),
        Ok(Err(e)) => Response::new_err(
            id,
            lsp_server::ErrorCode::InternalError as i32,
            e.to_string(),
        ),
        Err(e) => Response::new_err(id, lsp_server::ErrorCode::InvalidParams as i32, e),
    }
}

fn params<R: lsp_types::request::Request>(req: Request) -> Result<R::Params, String> {
    match req.extract::<R::Params>(R::METHOD) {
        Ok((_, p)) => Ok(p),
        Err(ExtractError::JsonError { error, .. }) => Err(error.to_string()),
        Err(ExtractError::MethodMismatch(r)) => Err(format!("not {}: {}", R::METHOD, r.method)),
    }
}

/// The hints whose type ends inside `range`, after the type, its hover as the tooltip.
fn inlay_hints(doc: &Doc, range: Range) -> Vec<InlayHint> {
    let (from, to) = (doc.offset(range.start), doc.offset(range.end));
    doc.found
        .iter()
        .filter(|f| from <= f.range.end && f.range.end <= to)
        .map(|f| InlayHint {
            position: doc.pos(f.range.end),
            label: InlayHintLabel::String(f.hint.clone()),
            kind: None,
            text_edits: None,
            tooltip: Some(InlayHintTooltip::MarkupContent(markdown(&f.hover))),
            padding_left: Some(true),
            padding_right: None,
            data: None,
        })
        .collect()
}

fn hover(doc: &Doc, at: Position) -> Option<Hover> {
    let f = ww_lsp_core::at(&doc.found, doc.offset(at))?;
    Some(Hover {
        contents: HoverContents::Markup(markdown(&f.hover)),
        range: Some(doc.range(&f.range)),
    })
}

/// The standard token type of each [TokenKind], in the order of [TokenKind::ALL]: a theme colours them without a
/// setting of its own.
fn token_type(kind: TokenKind) -> SemanticTokenType {
    match kind {
        TokenKind::ItemMacro => SemanticTokenType::MACRO,
        TokenKind::AccessMode => SemanticTokenType::KEYWORD,
        TokenKind::Attribute => SemanticTokenType::DECORATOR,
        TokenKind::Wrapper => SemanticTokenType::STRUCT,
        TokenKind::Alias => SemanticTokenType::TYPE,
        TokenKind::Unit => SemanticTokenType::ENUM_MEMBER,
        TokenKind::Prefix => SemanticTokenType::NUMBER,
        TokenKind::Origin => SemanticTokenType::ENUM,
    }
}

fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TokenKind::ALL.into_iter().map(token_type).collect(),
        // bit 0: a name of exact_units or ww_si, not WireWeaver syntax
        token_modifiers: vec![SemanticTokenModifier::DEFAULT_LIBRARY],
    }
}

/// The tokens that start inside `range` (all without one), relative to the one before as LSP encodes them.
fn semantic_tokens(doc: &Doc, range: Option<Range>) -> Vec<SemanticToken> {
    let (from, to) = match range {
        Some(r) => (doc.offset(r.start), doc.offset(r.end)),
        None => (0, usize::MAX),
    };
    let mut out = vec![];
    let mut prev = Position::new(0, 0);
    for t in doc
        .tokens
        .iter()
        .filter(|t| from <= t.range.start && t.range.start < to)
    {
        let (start, end) = (doc.pos(t.range.start), doc.pos(t.range.end));
        out.push(SemanticToken {
            delta_line: start.line - prev.line,
            delta_start: if start.line == prev.line {
                start.character - prev.character
            } else {
                start.character
            },
            // a token is one identifier, on one line
            length: end.character - start.character,
            token_type: t.kind.index(),
            token_modifiers_bitset: u32::from(t.kind.is_library()),
        });
        prev = start;
    }
    out
}

fn markdown(text: &str) -> MarkupContent {
    MarkupContent {
        kind: MarkupKind::Markdown,
        value: text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_server::{Connection, RequestId};
    use serde_json::json;

    fn id(n: i32) -> RequestId {
        RequestId::from(n)
    }

    const SRC: &str =
        "#[ww_trait]\ntrait Psu {\n    property!(ro v_out: Fixed<Volt, Milli, i16>);\n}\n";

    fn request(
        conn: &Connection,
        n: i32,
        method: &str,
        params: serde_json::Value,
    ) -> serde_json::Value {
        conn.sender
            .send(Message::Request(Request::new(id(n), method.into(), params)))
            .unwrap();
        loop {
            match conn.receiver.recv().unwrap() {
                Message::Response(r) if r.id == id(n) => return r.result.unwrap_or_default(),
                _ => {}
            }
        }
    }

    fn notify(conn: &Connection, method: &str, params: serde_json::Value) {
        conn.sender
            .send(Message::Notification(Notification::new(
                method.into(),
                params,
            )))
            .unwrap();
    }

    /// A whole session in memory: initialize, open, hints, hover, a change that doesn't parse, shutdown.
    #[test]
    fn session() {
        let (server, client) = Connection::memory();
        let thread = std::thread::spawn(move || serve(&server).unwrap());
        let init = request(&client, 1, "initialize", json!({ "capabilities": {} }));
        assert_eq!(init["capabilities"]["inlayHintProvider"], json!(true));
        assert_eq!(init["serverInfo"]["name"], json!("ww_lsp"));
        notify(&client, "initialized", json!({}));
        let uri = "file:///psu.rs";
        notify(
            &client,
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "rust", "version": 1, "text": SRC } }),
        );
        let whole =
            json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 9, "character": 0 } });
        let hints = request(
            &client,
            2,
            "textDocument/inlayHint",
            json!({ "textDocument": { "uri": uri }, "range": whole }),
        );
        assert_eq!(hints[0]["label"], json!("mV · i16 · ±32.767 V"));
        assert_eq!(hints[0]["position"], json!({ "line": 2, "character": 47 }));
        let hover = request(
            &client,
            3,
            "textDocument/hover",
            json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": 30 } }),
        );
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("- step: 0.001 V")
        );
        assert_eq!(
            hover["range"]["start"],
            json!({ "line": 2, "character": 24 })
        );
        // half typed: the hints of the last text that parsed stay
        notify(
            &client,
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "text": "#[ww_trait]\ntrait Psu {" }] }),
        );
        let hints = request(
            &client,
            4,
            "textDocument/inlayHint",
            json!({ "textDocument": { "uri": uri }, "range": whole }),
        );
        assert_eq!(hints.as_array().unwrap().len(), 1);
        request(&client, 5, "shutdown", json!(null));
        notify(&client, "exit", json!(null));
        thread.join().unwrap();
    }

    /// Semantic tokens over a whole session: the legend, the full file, a range, and a change that doesn't parse.
    #[test]
    fn semantic_tokens_session() {
        let (server, client) = Connection::memory();
        let thread = std::thread::spawn(move || serve(&server).unwrap());
        let init = request(&client, 1, "initialize", json!({ "capabilities": {} }));
        let provider = &init["capabilities"]["semanticTokensProvider"];
        assert_eq!(
            provider["legend"]["tokenTypes"],
            json!([
                "macro",
                "keyword",
                "decorator",
                "struct",
                "type",
                "enumMember",
                "number",
                "enum"
            ])
        );
        assert_eq!(
            provider["legend"]["tokenModifiers"],
            json!(["defaultLibrary"])
        );
        assert_eq!(provider["range"], json!(true));
        notify(&client, "initialized", json!({}));
        let uri = "file:///psu.rs";
        notify(
            &client,
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "rust", "version": 1, "text": SRC } }),
        );
        let full = request(
            &client,
            2,
            "textDocument/semanticTokens/full",
            json!({ "textDocument": { "uri": uri } }),
        );
        // ww_trait at 0:2; property at 2:4; ro at 2:14; Fixed at 2:24; Volt 2:30; Milli 2:36
        let want = json!([
            0, 2, 8, 2, 0, //
            2, 4, 8, 0, 0, //
            0, 10, 2, 1, 0, //
            0, 10, 5, 3, 1, //
            0, 6, 4, 5, 1, //
            0, 6, 5, 6, 1
        ]);
        assert_eq!(full["data"], want);
        let line2 =
            json!({ "start": { "line": 2, "character": 0 }, "end": { "line": 3, "character": 0 } });
        let range = request(
            &client,
            3,
            "textDocument/semanticTokens/range",
            json!({ "textDocument": { "uri": uri }, "range": line2 }),
        );
        assert_eq!(
            range["data"][0],
            json!(2),
            "the first token in the range, from line 0"
        );
        assert_eq!(range["data"].as_array().unwrap().len(), 25);
        // half typed: the tokens of the last text that parsed stay
        notify(
            &client,
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "text": "#[ww_trait]\ntrait Psu {" }] }),
        );
        let full = request(
            &client,
            4,
            "textDocument/semanticTokens/full",
            json!({ "textDocument": { "uri": uri } }),
        );
        assert_eq!(full["data"], want);
        request(&client, 5, "shutdown", json!(null));
        notify(&client, "exit", json!(null));
        thread.join().unwrap();
    }
}
