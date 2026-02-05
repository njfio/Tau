use std::fmt;

pub trait Component {
    fn render(&self, width: usize) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    content: String,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Component for Text {
    fn render(&self, width: usize) -> Vec<String> {
        wrap_text(&self.content, width)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderOp {
    Update { line: usize, content: String },
    ClearFrom { line: usize },
}

impl fmt::Display for RenderOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderOp::Update { line, content } => write!(f, "update({line}):{content}"),
            RenderOp::ClearFrom { line } => write!(f, "clear_from({line})"),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct DiffRenderer {
    previous: Vec<String>,
}

impl DiffRenderer {
    pub fn new() -> Self {
        Self {
            previous: Vec::new(),
        }
    }

    pub fn diff(&mut self, next: Vec<String>) -> Vec<RenderOp> {
        let mut operations = Vec::new();
        let max_len = self.previous.len().max(next.len());

        for index in 0..max_len {
            match (self.previous.get(index), next.get(index)) {
                (Some(old), Some(new)) if old != new => operations.push(RenderOp::Update {
                    line: index,
                    content: new.clone(),
                }),
                (None, Some(new)) => operations.push(RenderOp::Update {
                    line: index,
                    content: new.clone(),
                }),
                _ => {}
            }
        }

        if next.len() < self.previous.len() {
            operations.push(RenderOp::ClearFrom { line: next.len() });
        }

        self.previous = next;
        operations
    }

    pub fn snapshot(&self) -> &[String] {
        &self.previous
    }
}

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();

    for raw_line in text.lines() {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            let required = if current.is_empty() {
                word.len()
            } else {
                current.len() + 1 + word.len()
            };

            if required <= width {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
                continue;
            }

            if !current.is_empty() {
                lines.push(current);
                current = String::new();
            }

            if word.len() > width {
                let mut start = 0;
                let bytes = word.as_bytes();
                while start < bytes.len() {
                    let end = (start + width).min(bytes.len());
                    let segment = &word[start..end];
                    lines.push(segment.to_string());
                    start = end;
                }
            } else {
                current.push_str(word);
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::{wrap_text, DiffRenderer, RenderOp, Text};
    use crate::Component;

    #[test]
    fn wraps_text_to_width() {
        let lines = wrap_text("one two three", 7);
        assert_eq!(lines, vec!["one two", "three"]);
    }

    #[test]
    fn wraps_long_word() {
        let lines = wrap_text("abcdefghij", 4);
        assert_eq!(lines, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn text_component_renders_with_wrap() {
        let component = Text::new("hello world");
        assert_eq!(component.render(5), vec!["hello", "world"]);
    }

    #[test]
    fn renderer_outputs_only_changed_lines() {
        let mut renderer = DiffRenderer::new();
        let first = renderer.diff(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            first,
            vec![
                RenderOp::Update {
                    line: 0,
                    content: "a".to_string()
                },
                RenderOp::Update {
                    line: 1,
                    content: "b".to_string()
                }
            ]
        );

        let second = renderer.diff(vec!["a".to_string(), "c".to_string()]);
        assert_eq!(
            second,
            vec![RenderOp::Update {
                line: 1,
                content: "c".to_string()
            }]
        );

        let third = renderer.diff(vec!["a".to_string()]);
        assert_eq!(third, vec![RenderOp::ClearFrom { line: 1 }]);
    }
}
