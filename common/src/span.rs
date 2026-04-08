/// Represents a source location span (byte offsets into the source text).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// Create a span representing an unknown source location.
    pub fn unknown() -> Self {
        Span { start: 0, end: 0 }
    }

    /// Returns true if this span represents an unknown/unset location.
    pub fn is_unknown(&self) -> bool {
        self.start == 0 && self.end == 0
    }

    /// Merge two spans into one that covers both.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Returns the length of this span in bytes.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the span has zero length.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert a byte offset to a (line, column) pair (both 1-based).
    /// Returns `None` if the offset is beyond the end of the source.
    pub fn offset_to_line_col(source: &str, offset: usize) -> Option<(usize, usize)> {
        if offset > source.len() {
            return None;
        }
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in source.char_indices() {
            if i == offset {
                return Some((line, col));
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        // offset == source.len() (one past end)
        Some((line, col))
    }

    /// Get the (line, column) of the start of this span.
    pub fn start_line_col(&self, source: &str) -> Option<(usize, usize)> {
        Self::offset_to_line_col(source, self.start)
    }

    /// Get the (line, column) of the end of this span.
    pub fn end_line_col(&self, source: &str) -> Option<(usize, usize)> {
        Self::offset_to_line_col(source, self.end)
    }

    /// Extract the source text covered by this span.
    /// Returns `None` if the span is out of bounds.
    pub fn source_text<'a>(&self, source: &'a str) -> Option<&'a str> {
        source.get(self.start..self.end)
    }

    /// Get the full line of source text that contains the start of this span.
    /// Returns the line text, the 1-based line number, and the 1-based column offset.
    pub fn source_line<'a>(&self, source: &'a str) -> Option<(&'a str, usize, usize)> {
        let (line_num, col) = self.start_line_col(source)?;
        let line_start = source[..self.start]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        let line_end = source[self.start..]
            .find('\n')
            .map(|pos| self.start + pos)
            .unwrap_or(source.len());
        let line_text = &source[line_start..line_end];
        Some((line_text, line_num, col))
    }

    /// Format a rich diagnostic message with source context.
    ///
    /// Produces output like:
    /// ```text
    /// error: Undefined variable: x
    ///   --> source.roga:5:12
    ///    |
    ///  5 |     result = x + y
    ///    |              ^
    /// ```
    pub fn format_diagnostic(
        &self,
        source: &str,
        filename: &str,
        level: DiagnosticLevel,
        message: &str,
    ) -> String {
        let mut out = String::new();

        // Header line: "error: message"
        out.push_str(&format!("{level}: {message}\n"));

        if self.is_unknown() {
            return out;
        }

        if let Some((line_text, line_num, col)) = self.source_line(source) {
            let line_num_str = line_num.to_string();
            let gutter_width = line_num_str.len();

            // Location line: "  --> file:line:col"
            out.push_str(&format!(
                "{:>gutter_width$}--> {filename}:{line_num}:{col}\n",
                " ",
            ));

            // Blank gutter line
            out.push_str(&format!("{:>gutter_width$} |\n", " "));

            // Source line with gutter
            out.push_str(&format!("{line_num_str} | {line_text}\n"));

            // Underline
            let underline_len = if !self.is_empty() {
                // Clamp underline to not exceed the line
                let max_len = line_text.len().saturating_sub(col.saturating_sub(1));
                self.len().min(max_len).max(1)
            } else {
                1
            };
            out.push_str(&format!(
                "{:>gutter_width$} | {:>col_pad$}{}\n",
                " ",
                "",
                "^".repeat(underline_len),
                col_pad = col.saturating_sub(1),
            ));
        }

        out
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A wrapper that associates a value with a source span.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }

    pub fn unknown(node: T) -> Self {
        Spanned {
            node,
            span: Span::unknown(),
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }

    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            node: &self.node,
            span: self.span,
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.node.fmt(f)
    }
}

/// Severity level for diagnostic messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Hint,
}

impl std::fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
            DiagnosticLevel::Info => write!(f, "info"),
            DiagnosticLevel::Hint => write!(f, "hint"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(5, 10);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 10);
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
        assert!(!span.is_unknown());
    }

    #[test]
    fn test_span_unknown() {
        let span = Span::unknown();
        assert!(span.is_unknown());
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_merge() {
        let a = Span::new(5, 10);
        let b = Span::new(8, 15);
        let merged = a.merge(b);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_offset_to_line_col() {
        let source = "hello\nworld\nfoo";
        // 'h' is at offset 0 => line 1, col 1
        assert_eq!(Span::offset_to_line_col(source, 0), Some((1, 1)));
        // 'e' at offset 1 => line 1, col 2
        assert_eq!(Span::offset_to_line_col(source, 1), Some((1, 2)));
        // '\n' at offset 5 => line 1, col 6
        assert_eq!(Span::offset_to_line_col(source, 5), Some((1, 6)));
        // 'w' at offset 6 => line 2, col 1
        assert_eq!(Span::offset_to_line_col(source, 6), Some((2, 1)));
        // 'f' at offset 12 => line 3, col 1
        assert_eq!(Span::offset_to_line_col(source, 12), Some((3, 1)));
        // past end
        assert_eq!(Span::offset_to_line_col(source, 100), None);
    }

    #[test]
    fn test_source_text() {
        let source = "hello world";
        let span = Span::new(6, 11);
        assert_eq!(span.source_text(source), Some("world"));
    }

    #[test]
    fn test_source_line() {
        let source = "line one\nline two\nline three";
        let span = Span::new(14, 17); // "two" on line 2
        let (line_text, line_num, col) = span.source_line(source).unwrap();
        assert_eq!(line_text, "line two");
        assert_eq!(line_num, 2);
        assert_eq!(col, 6);
    }

    #[test]
    fn test_format_diagnostic() {
        let source = "let x = 1\nlet y = x + z\nlet w = 3";
        // 'z' is at offset 23 (line 2, col 14)
        let span = Span::new(23, 24);
        let output = span.format_diagnostic(
            source,
            "test.roga",
            DiagnosticLevel::Error,
            "Undefined variable: z",
        );
        assert!(output.contains("error: Undefined variable: z"));
        assert!(output.contains("--> test.roga:2:14"));
        assert!(output.contains("let y = x + z"));
        assert!(output.contains("^"));
    }

    #[test]
    fn test_format_diagnostic_unknown_span() {
        let output = Span::unknown().format_diagnostic(
            "source",
            "test.roga",
            DiagnosticLevel::Error,
            "Something went wrong",
        );
        assert_eq!(output, "error: Something went wrong\n");
    }

    #[test]
    fn test_spanned_wrapper() {
        let spanned = Spanned::new(42, Span::new(0, 2));
        assert_eq!(spanned.node, 42);
        assert_eq!(spanned.span, Span::new(0, 2));

        let mapped = spanned.map(|n| n.to_string());
        assert_eq!(mapped.node, "42");
        assert_eq!(mapped.span, Span::new(0, 2));
    }

    #[test]
    fn test_spanned_unknown() {
        let spanned = Spanned::unknown("hello");
        assert!(spanned.span.is_unknown());
    }

    #[test]
    fn test_span_display() {
        let span = Span::new(5, 10);
        assert_eq!(format!("{span}"), "5..10");
    }

    #[test]
    fn test_spanned_display() {
        let spanned = Spanned::new("hello", Span::new(0, 5));
        assert_eq!(format!("{spanned}"), "hello");
    }

    #[test]
    fn test_diagnostic_level_display() {
        assert_eq!(format!("{}", DiagnosticLevel::Error), "error");
        assert_eq!(format!("{}", DiagnosticLevel::Warning), "warning");
        assert_eq!(format!("{}", DiagnosticLevel::Info), "info");
        assert_eq!(format!("{}", DiagnosticLevel::Hint), "hint");
    }

    #[test]
    fn test_multi_char_underline() {
        let source = "let result = longVariable + other";
        // "longVariable" spans from offset 13 to 25
        let span = Span::new(13, 25);
        let output = span.format_diagnostic(
            source,
            "test.roga",
            DiagnosticLevel::Warning,
            "Unused variable",
        );
        assert!(output.contains("^^^^^^^^^^^^")); // 12 carets for "longVariable"
    }

    #[test]
    fn test_first_line_diagnostic() {
        let source = "badVar + 1";
        let span = Span::new(0, 6);
        let output = span.format_diagnostic(
            source,
            "test.roga",
            DiagnosticLevel::Error,
            "Undefined variable: badVar",
        );
        assert!(output.contains("--> test.roga:1:1"));
        assert!(output.contains("badVar + 1"));
        assert!(output.contains("^^^^^^"));
    }
}
