use leptos::prelude::*;

pub(crate) fn contains_markdown_contract_syntax(content: &str) -> bool {
    content.contains("```")
        || content.starts_with('#')
        || content.contains("\n#")
        || content.starts_with("- ")
        || content.contains("\n- ")
        || content.contains("1. ")
        || content.contains("**")
        || content.contains('`')
        || content.contains("](")
        || content.contains("Immediate next checks:")
        || content.contains("Explicit risks:")
        || (content.contains('|') && content.contains("\n|---"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TauOpsDashboardChatMarkdownInline {
    Text(String),
    Strong(String),
    Code(String),
    Link { label: String, href: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TauOpsDashboardChatMarkdownBlock {
    Paragraph(Vec<TauOpsDashboardChatMarkdownInline>),
    Heading {
        level: u8,
        content: Vec<TauOpsDashboardChatMarkdownInline>,
    },
    UnorderedList(Vec<Vec<TauOpsDashboardChatMarkdownInline>>),
    OrderedList(Vec<Vec<TauOpsDashboardChatMarkdownInline>>),
    CodeBlock {
        language: String,
        code: String,
    },
    Table {
        header: Vec<Vec<TauOpsDashboardChatMarkdownInline>>,
        rows: Vec<Vec<Vec<TauOpsDashboardChatMarkdownInline>>>,
    },
}

fn sanitize_markdown_href(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("https://")
        || trimmed.starts_with("http://")
        || trimmed.starts_with('/')
        || trimmed.starts_with('#')
    {
        trimmed.to_string()
    } else {
        "#".to_string()
    }
}

fn parse_chat_markdown_inline(content: &str) -> Vec<TauOpsDashboardChatMarkdownInline> {
    let mut remaining = content;
    let mut parts = Vec::new();
    while !remaining.is_empty() {
        let next_strong = remaining.find("**");
        let next_code = remaining.find('`');
        let next_link = remaining.find('[');
        let next = [next_strong, next_code, next_link]
            .into_iter()
            .flatten()
            .min();
        let Some(next_index) = next else {
            parts.push(TauOpsDashboardChatMarkdownInline::Text(
                remaining.to_string(),
            ));
            break;
        };
        if next_index > 0 {
            parts.push(TauOpsDashboardChatMarkdownInline::Text(
                remaining[..next_index].to_string(),
            ));
            remaining = &remaining[next_index..];
            continue;
        }
        if let Some(after_open) = remaining.strip_prefix("**") {
            if let Some(close_index) = after_open.find("**") {
                let value = after_open[..close_index].trim();
                if !value.is_empty() {
                    parts.push(TauOpsDashboardChatMarkdownInline::Strong(value.to_string()));
                }
                remaining = &after_open[close_index + 2..];
                continue;
            }
        }
        if let Some(after_tick) = remaining.strip_prefix('`') {
            if let Some(close_index) = after_tick.find('`') {
                parts.push(TauOpsDashboardChatMarkdownInline::Code(
                    after_tick[..close_index].to_string(),
                ));
                remaining = &after_tick[close_index + 1..];
                continue;
            }
        }
        if remaining.starts_with('[') {
            if let Some(label_end) = remaining.find("](") {
                let href_start = label_end + 2;
                if let Some(href_end_relative) = remaining[href_start..].find(')') {
                    let href_end = href_start + href_end_relative;
                    let label = remaining[1..label_end].trim();
                    let href = sanitize_markdown_href(&remaining[href_start..href_end]);
                    if !label.is_empty() {
                        parts.push(TauOpsDashboardChatMarkdownInline::Link {
                            label: label.to_string(),
                            href,
                        });
                        remaining = &remaining[href_end + 1..];
                        continue;
                    }
                }
            }
        }
        parts.push(TauOpsDashboardChatMarkdownInline::Text(
            remaining[..1].to_string(),
        ));
        remaining = &remaining[1..];
    }
    if parts.is_empty() {
        parts.push(TauOpsDashboardChatMarkdownInline::Text(String::new()));
    }
    parts
}

fn markdown_heading_line(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|value| *value == '#').count();
    if !(1..=6).contains(&level) || !trimmed[level..].starts_with(' ') {
        return None;
    }
    let heading = trimmed[level..].trim();
    if heading.is_empty() {
        return None;
    }
    Some((level as u8, heading.to_string()))
}

fn ordered_list_item(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let dot_index = trimmed.find('.')?;
    if dot_index == 0
        || !trimmed[..dot_index]
            .chars()
            .all(|value| value.is_ascii_digit())
    {
        return None;
    }
    let after_dot = &trimmed[dot_index + 1..];
    let item = after_dot.strip_prefix(' ')?;
    if item.trim().is_empty() {
        None
    } else {
        Some(item.trim())
    }
}

fn unordered_list_item(line: &str) -> Option<&str> {
    line.trim_start()
        .strip_prefix("- ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn is_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains('|')
        && trimmed
            .chars()
            .all(|value| matches!(value, '|' | '-' | ':' | ' '))
        && trimmed.contains("---")
}

fn parse_table_cells(line: &str) -> Vec<Vec<TauOpsDashboardChatMarkdownInline>> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| parse_chat_markdown_inline(cell.trim()))
        .collect()
}

fn markdown_block_starts(lines: &[&str], index: usize) -> bool {
    let line = lines[index].trim_start();
    line.starts_with("```")
        || markdown_heading_line(line).is_some()
        || ordered_list_item(line).is_some()
        || unordered_list_item(line).is_some()
        || (index + 1 < lines.len()
            && line.contains('|')
            && is_table_separator_line(lines[index + 1]))
}

fn find_ordered_marker(content: &str, number: usize, from_index: usize) -> Option<usize> {
    let marker = format!("{number}. ");
    let mut search_from = from_index;
    while search_from < content.len() {
        let relative = content[search_from..].find(&marker)?;
        let index = search_from + relative;
        let boundary = index == 0
            || content[..index]
                .chars()
                .next_back()
                .is_some_and(|value| value.is_whitespace() || matches!(value, ':' | ';'));
        if boundary {
            return Some(index);
        }
        search_from = index + marker.len();
    }
    None
}

fn parse_inline_ordered_markdown_items(
    content: &str,
) -> Vec<Vec<TauOpsDashboardChatMarkdownInline>> {
    let mut starts = Vec::new();
    let mut search_from = 0;
    for number in 1..=20 {
        let Some(marker_index) = find_ordered_marker(content, number, search_from) else {
            break;
        };
        let marker = format!("{number}. ");
        let content_start = marker_index + marker.len();
        starts.push((marker_index, content_start));
        search_from = content_start;
    }
    starts
        .iter()
        .enumerate()
        .filter_map(|(index, (_, content_start))| {
            let content_end = starts
                .get(index + 1)
                .map(|(next_marker_index, _)| *next_marker_index)
                .unwrap_or(content.len());
            let item = content[*content_start..content_end].trim();
            (!item.is_empty()).then(|| parse_chat_markdown_inline(item))
        })
        .collect()
}

fn parse_inline_unordered_markdown_items(
    content: &str,
) -> Vec<Vec<TauOpsDashboardChatMarkdownInline>> {
    let trimmed = content.trim();
    let item_source = if let Some(stripped) = trimmed.strip_prefix("- ") {
        stripped
    } else if trimmed.contains(" - ") {
        trimmed
    } else {
        return Vec::new();
    };
    item_source
        .split(" - ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(parse_chat_markdown_inline)
        .collect()
}

fn parse_operator_response_markdown_blocks(
    content: &str,
) -> Option<Vec<TauOpsDashboardChatMarkdownBlock>> {
    let checks_marker = "Immediate next checks:";
    let risks_marker = "Explicit risks:";
    let checks_index = content.find(checks_marker);
    let risks_index = content.find(risks_marker);
    if checks_index.is_none() && risks_index.is_none() {
        return None;
    }
    let first_marker = [checks_index, risks_index]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(0);
    let mut blocks = Vec::new();
    let intro = content[..first_marker].trim();
    if !intro.is_empty() {
        blocks.push(TauOpsDashboardChatMarkdownBlock::Paragraph(
            parse_chat_markdown_inline(intro),
        ));
    }
    if let Some(index) = checks_index {
        let start = index + checks_marker.len();
        let end = risks_index
            .filter(|risk_index| *risk_index > index)
            .unwrap_or(content.len());
        let checks = content[start..end].trim();
        if !checks.is_empty() {
            blocks.push(TauOpsDashboardChatMarkdownBlock::Heading {
                level: 5,
                content: parse_chat_markdown_inline("Immediate next checks"),
            });
            let items = parse_inline_ordered_markdown_items(checks);
            if items.is_empty() {
                blocks.push(TauOpsDashboardChatMarkdownBlock::Paragraph(
                    parse_chat_markdown_inline(checks),
                ));
            } else {
                blocks.push(TauOpsDashboardChatMarkdownBlock::OrderedList(items));
            }
        }
    }
    if let Some(index) = risks_index {
        let risks = content[index + risks_marker.len()..].trim();
        if !risks.is_empty() {
            blocks.push(TauOpsDashboardChatMarkdownBlock::Heading {
                level: 5,
                content: parse_chat_markdown_inline("Explicit risks"),
            });
            let items = parse_inline_unordered_markdown_items(risks);
            if items.is_empty() {
                blocks.push(TauOpsDashboardChatMarkdownBlock::Paragraph(
                    parse_chat_markdown_inline(risks),
                ));
            } else {
                blocks.push(TauOpsDashboardChatMarkdownBlock::UnorderedList(items));
            }
        }
    }
    (!blocks.is_empty()).then_some(blocks)
}

pub(crate) fn parse_chat_markdown_blocks(content: &str) -> Vec<TauOpsDashboardChatMarkdownBlock> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return vec![TauOpsDashboardChatMarkdownBlock::Paragraph(vec![
            TauOpsDashboardChatMarkdownInline::Text(String::new()),
        ])];
    }
    if let Some(blocks) = parse_operator_response_markdown_blocks(trimmed) {
        return blocks;
    }
    let lines: Vec<&str> = trimmed.lines().collect();
    let mut blocks = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if line.is_empty() {
            index += 1;
            continue;
        }
        if let Some(language) = line.strip_prefix("```") {
            index += 1;
            let mut code_lines = Vec::new();
            while index < lines.len() {
                if lines[index].trim_start().starts_with("```") {
                    index += 1;
                    break;
                }
                code_lines.push(lines[index]);
                index += 1;
            }
            blocks.push(TauOpsDashboardChatMarkdownBlock::CodeBlock {
                language: if language.trim().is_empty() {
                    "plain".to_string()
                } else {
                    language.trim().to_string()
                },
                code: code_lines.join("\n").trim().to_string(),
            });
            continue;
        }
        if let Some((level, heading)) = markdown_heading_line(line) {
            blocks.push(TauOpsDashboardChatMarkdownBlock::Heading {
                level,
                content: parse_chat_markdown_inline(&heading),
            });
            index += 1;
            continue;
        }
        if index + 1 < lines.len()
            && line.contains('|')
            && is_table_separator_line(lines[index + 1])
        {
            let header = parse_table_cells(line);
            index += 2;
            let mut rows = Vec::new();
            while index < lines.len() {
                let row = lines[index].trim();
                if row.is_empty() || !row.contains('|') || markdown_block_starts(&lines, index) {
                    break;
                }
                rows.push(parse_table_cells(row));
                index += 1;
            }
            blocks.push(TauOpsDashboardChatMarkdownBlock::Table { header, rows });
            continue;
        }
        if let Some(item) = unordered_list_item(line) {
            let mut items = vec![parse_chat_markdown_inline(item)];
            index += 1;
            while index < lines.len() {
                let Some(item) = unordered_list_item(lines[index]) else {
                    break;
                };
                items.push(parse_chat_markdown_inline(item));
                index += 1;
            }
            blocks.push(TauOpsDashboardChatMarkdownBlock::UnorderedList(items));
            continue;
        }
        if let Some(item) = ordered_list_item(line) {
            let mut items = vec![parse_chat_markdown_inline(item)];
            index += 1;
            while index < lines.len() {
                let Some(item) = ordered_list_item(lines[index]) else {
                    break;
                };
                items.push(parse_chat_markdown_inline(item));
                index += 1;
            }
            blocks.push(TauOpsDashboardChatMarkdownBlock::OrderedList(items));
            continue;
        }
        let mut paragraph_lines = vec![line.to_string()];
        index += 1;
        while index < lines.len() {
            let next_line = lines[index].trim();
            if next_line.is_empty() || markdown_block_starts(&lines, index) {
                break;
            }
            paragraph_lines.push(next_line.to_string());
            index += 1;
        }
        blocks.push(TauOpsDashboardChatMarkdownBlock::Paragraph(
            parse_chat_markdown_inline(&paragraph_lines.join(" ")),
        ));
    }
    if blocks.is_empty() {
        blocks.push(TauOpsDashboardChatMarkdownBlock::Paragraph(
            parse_chat_markdown_inline(trimmed),
        ));
    }
    blocks
}

fn render_chat_markdown_inlines(inlines: Vec<TauOpsDashboardChatMarkdownInline>) -> impl IntoView {
    inlines
        .into_iter()
        .map(|inline| match inline {
            TauOpsDashboardChatMarkdownInline::Text(value) => {
                view! { <span>{value}</span> }.into_any()
            }
            TauOpsDashboardChatMarkdownInline::Strong(value) => {
                view! { <strong>{value}</strong> }.into_any()
            }
            TauOpsDashboardChatMarkdownInline::Code(value) => {
                view! { <code data-markdown-inline-code="true">{value}</code> }.into_any()
            }
            TauOpsDashboardChatMarkdownInline::Link { label, href } => view! {
                <a data-markdown-link="true" href=href rel="noreferrer">{label}</a>
            }
            .into_any(),
        })
        .collect_view()
}

pub(crate) fn render_chat_markdown_blocks(
    blocks: Vec<TauOpsDashboardChatMarkdownBlock>,
    primary_code_block_id: Option<String>,
) -> impl IntoView {
    let mut primary_code_block_id = primary_code_block_id;
    blocks
        .into_iter()
        .enumerate()
        .map(move |(block_index, block)| {
            let block_index_attr = block_index.to_string();
            match block {
                TauOpsDashboardChatMarkdownBlock::Paragraph(inlines) => view! {
                    <p data-markdown-block="paragraph" data-markdown-block-index=block_index_attr>
                        {render_chat_markdown_inlines(inlines)}
                    </p>
                }
                .into_any(),
                TauOpsDashboardChatMarkdownBlock::Heading { level, content } => {
                    let heading_level_attr = level.to_string();
                    view! {
                        <h4
                            data-markdown-block="heading"
                            data-markdown-heading-level=heading_level_attr
                            data-markdown-block-index=block_index_attr
                        >
                            {render_chat_markdown_inlines(content)}
                        </h4>
                    }
                    .into_any()
                }
                TauOpsDashboardChatMarkdownBlock::UnorderedList(items) => view! {
                    <ul
                        data-markdown-block="list"
                        data-markdown-list="unordered"
                        data-markdown-block-index=block_index_attr
                    >
                        {items
                            .into_iter()
                            .map(|item| {
                                view! {
                                    <li data-markdown-list-item="unordered">
                                        {render_chat_markdown_inlines(item)}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any(),
                TauOpsDashboardChatMarkdownBlock::OrderedList(items) => view! {
                    <ol
                        data-markdown-block="list"
                        data-markdown-list="ordered"
                        data-markdown-block-index=block_index_attr
                    >
                        {items
                            .into_iter()
                            .map(|item| {
                                view! {
                                    <li data-markdown-list-item="ordered">
                                        {render_chat_markdown_inlines(item)}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ol>
                }
                .into_any(),
                TauOpsDashboardChatMarkdownBlock::CodeBlock { language, code } => {
                    let code_block_id = primary_code_block_id.take().unwrap_or_else(|| {
                        format!("tau-ops-chat-code-block-extra-{block_index}")
                    });
                    let code_attribute = code.clone();
                    view! {
                        <pre
                            id=code_block_id
                            data-markdown-block="code"
                            data-code-block="true"
                            data-language=language
                            data-code=code_attribute
                            data-markdown-block-index=block_index_attr
                        >
                            {code}
                        </pre>
                    }
                    .into_any()
                }
                TauOpsDashboardChatMarkdownBlock::Table { header, rows } => view! {
                    <table
                        data-markdown-block="table"
                        data-markdown-table="true"
                        data-markdown-block-index=block_index_attr
                    >
                        <thead>
                            <tr>
                                {header
                                    .into_iter()
                                    .map(|cell| {
                                        view! { <th scope="col">{render_chat_markdown_inlines(cell)}</th> }
                                    })
                                    .collect_view()}
                            </tr>
                        </thead>
                        <tbody>
                            {rows
                                .into_iter()
                                .map(|row| {
                                    view! {
                                        <tr>
                                            {row
                                                .into_iter()
                                                .map(|cell| {
                                                    view! { <td>{render_chat_markdown_inlines(cell)}</td> }
                                                })
                                                .collect_view()}
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                }
                .into_any(),
            }
        })
        .collect_view()
}

pub(crate) fn extract_first_fenced_code_block(content: &str) -> Option<(String, String)> {
    let fence_start = content.find("```")?;
    let after_open_fence = &content[fence_start + 3..];
    let fence_end = after_open_fence.find("```")?;
    let fenced_block = &after_open_fence[..fence_end];
    let (language, code) = if let Some((language, code)) = fenced_block.split_once('\n') {
        (language.trim(), code.trim())
    } else {
        ("plain", fenced_block.trim())
    };
    if code.is_empty() {
        return None;
    }
    let language = if language.is_empty() {
        "plain"
    } else {
        language
    };
    Some((language.to_string(), code.to_string()))
}

pub(crate) fn extract_assistant_stream_tokens(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}
