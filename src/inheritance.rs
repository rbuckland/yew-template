use html5ever::tokenizer::{Tokenizer, TokenizerOpts, BufferQueue};
use crate::*;

/// Parses an HTML string and returns the raw parts (same mechanics as read_template).
fn parse_html(content: String, args: &Args) -> Vec<HtmlPartWithLine> {
    let mut html_parts = Vec::new();
    let html_sink = HtmlSink { html_parts: &mut html_parts, opened_elements: Vec::new(), args };
    let mut tokenizer = Tokenizer::new(html_sink, TokenizerOpts::default());
    let mut bq = BufferQueue::new();
    bq.push_back(content.into());
    let _ = tokenizer.feed(&mut bq);
    tokenizer.end();
    html_parts
}

/// Walks a flat list of root-level parts and extracts every `<block name="…">`
/// into a map of block-name → children.  Everything else (including `<extends/>`)
/// is silently dropped — only named blocks matter in a child template.
fn collect_child_blocks(children: Vec<HtmlPartWithLine>) -> HashMap<String, Vec<HtmlPartWithLine>> {
    let mut blocks: HashMap<String, Vec<HtmlPartWithLine>> = HashMap::new();
    for item in children {
        match item.part {
            HtmlPart::Element(el) if el.name == "block" => {
                let name = el.open_attrs.iter()
                    .find(|(k, _)| k == "name")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                blocks.insert(name, el.children);
            }
            _ => {} // <extends/> and loose text/elements at child-root level are ignored
        }
    }
    blocks
}

/// Recursively replaces every `<block name="N">default</block>` in the tree with
/// the child's override content, or the base default when no override exists.
/// The `<block>` wrapper itself is removed; its content is inlined in place.
fn substitute_blocks(mut el: Element, overrides: &HashMap<String, Vec<HtmlPartWithLine>>) -> Element {
    let old_children = std::mem::take(&mut el.children);
    let mut new_children: Vec<HtmlPartWithLine> = Vec::new();

    for child in old_children {
        let line = child.line;
        match child.part {
            HtmlPart::Element(block_el) if block_el.name == "block" => {
                let name = block_el.open_attrs.iter()
                    .find(|(k, _)| k == "name")
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("");
                // Use child override when present, otherwise the base default content.
                let content = overrides.get(name).cloned().unwrap_or(block_el.children);
                new_children.extend(content);
            }
            HtmlPart::Element(child_el) => {
                new_children.push(HtmlPartWithLine {
                    part: HtmlPart::Element(substitute_blocks(child_el, overrides)),
                    line,
                });
            }
            other => new_children.push(HtmlPartWithLine { part: other, line }),
        }
    }
    el.children = new_children;
    el
}

/// Called from `read_template` after the template has been parsed and cleaned.
///
/// If the root's direct children contain `<extends src="path/to/base.html"/>`,
/// this function:
///   1. Extracts all `<block name="…">` overrides from the child template.
///   2. Loads and parses the parent template (resolving its own inheritance first).
///   3. Walks the parent tree and substitutes each `<block>` region with the
///      child's override, or keeps the base default when no override is supplied.
///
/// Returns the fully-resolved element tree ready for code generation.
/// If no `<extends/>` is present the root is returned unchanged.
pub(crate) fn resolve_inheritance(mut root: Element, args: &Args) -> Element {
    // Find an <extends src="…"/> among the root's direct children.
    let Some(pos) = root.children.iter().position(|c| {
        matches!(&c.part, HtmlPart::Element(el) if el.name == "extends")
    }) else {
        return root;
    };

    let extends_item = root.children.remove(pos);
    let extends_el = match extends_item.part {
        HtmlPart::Element(el) => el,
        _ => unreachable!(),
    };

    if !extends_el.self_closing {
        abort!(
            args.path_span,
            "<extends> must be self-closing: <extends src=\"path/to/base.html\"/>"
        );
    }

    // Guard against a second <extends> tag in the same template.
    if root.children.iter().any(|c| matches!(&c.part, HtmlPart::Element(el) if el.name == "extends")) {
        abort!(args.path_span, "A template may only have one <extends/> tag");
    }

    let src = extends_el.open_attrs.iter()
        .find(|(k, _)| k == "src")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| abort!(args.path_span, "<extends/> is missing the required 'src' attribute"));

    // Collect child block overrides from everything left in root.children.
    let child_blocks = collect_child_blocks(std::mem::take(&mut root.children));

    // Load the parent template.
    let parent_path = format!("{}{}", args.config.template_directory, src);
    let parent_content = match std::fs::read_to_string(&parent_path) {
        Ok(c) => c,
        Err(e) => abort!(args.path_span, "Failed to read parent template '{}': {}", parent_path, e),
    };

    let mut parent_root = Element {
        name: String::new(),
        open_attrs: Vec::new(),
        close_attrs: Vec::new(),
        self_closing: false,
        children: parse_html(parent_content, args),
    };
    parent_root.clean_text();

    // Recursively resolve the parent's own inheritance before substituting.
    let parent_root = resolve_inheritance(parent_root, args);

    // Substitute the child's block overrides into the resolved parent tree.
    substitute_blocks(parent_root, &child_blocks)
}
