use std::backtrace::{Backtrace, BacktraceStatus};

#[derive(Debug)]
pub struct StackTrace {
    pub frames: Vec<StackTraceFrame>,
}

#[derive(Debug)]
pub struct StackTraceFrame {
    pub func: String,
    pub file: String,
    pub line: u32,
}

impl StackTrace {
    /// parse [`Backtrace`]'s debug output
    pub fn parse(backtrace: &Backtrace) -> Option<Self> {
        if backtrace.status() != BacktraceStatus::Captured {
            return None;
        }

        let debug = format!("{:?}", backtrace);

        match Self::parse_debug_str(&debug) {
            Some(mut stacktrace) => {
                stacktrace.nomalize();
                Some(stacktrace)
            }
            None => None,
        }
    }

    /// parse [`Backtrace`]'s debug output
    pub fn parse_debug_str(debug: &str) -> Option<Self> {
        const LEADING: &str = "Backtrace ";

        // Extract the content inside "Backtrace [...]"
        let debug = debug.trim();

        // Find the start of the frames list
        if !debug.starts_with(LEADING) {
            return None;
        }

        // Skip "Backtrace " part
        let frames_part = &debug[LEADING.len()..].trim();

        // Check if it starts with '[' and ends with ']'
        if !frames_part.starts_with('[') || !frames_part.ends_with(']') {
            return None;
        }

        // Extract content between outer brackets
        let content = &frames_part[1..frames_part.len() - 1];

        let mut frames = Vec::new();
        let mut brace_depth = 0;
        let mut current_frame_start = 0;
        let mut chars = content.char_indices().peekable();

        while let Some((i, ch)) = chars.next() {
            match ch {
                '{' => {
                    if brace_depth == 0 {
                        current_frame_start = i;
                    }
                    brace_depth += 1;
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        // Found a complete frame
                        let frame_str = &content[current_frame_start..=i];
                        if let Some(frame) = parse_single_frame(frame_str) {
                            frames.push(frame);
                        }
                    }
                }
                _ => {}
            }
        }

        if frames.is_empty() {
            None
        } else {
            let stacktrace = StackTrace { frames };
            Some(stacktrace)
        }
    }

    /// Normalize stacktrace
    ///
    /// * Remove leading frames owned by [`Backtrace`]
    fn nomalize(&mut self) {
        //  * Remove leading frames owned by [`Backtrace`]
        loop {
            if let Some(first) = self.frames.first() {
                if first.func.starts_with("std::backtrace")
                    || first.func.starts_with("backerror::located_error")
                {
                    self.frames.remove(0);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}

/// Parse a single frame from the format: { fn: "...", file: "...", line: ... }
fn parse_single_frame(frame_str: &str) -> Option<StackTraceFrame> {
    // Remove the surrounding braces
    let trimmed = frame_str.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1].trim();

    // We'll extract values from the key-value pairs
    let func;
    let mut file = String::new();
    let mut line = 0u32;

    // Look for fn, file, and line in the string
    // Format: { fn: "func_name", file: "file_path", line: 123 }

    // Extract function name
    if let Some(func_match) = find_key_value(inner, r#"fn:"#) {
        func = func_match;
    } else {
        return None;
    }

    // Extract file name
    if let Some(file_match) = find_key_value(inner, r#"file:"#) {
        file = file_match;
    }

    // Extract line number
    if let Some(line_str) = find_key_value(inner, r#"line:"#) {
        if let Ok(parsed_line) = line_str.parse::<u32>() {
            line = parsed_line;
        }
    }

    Some(StackTraceFrame { func, file, line })
}

/// Find the value for a given key in a string of format `key: "value"` or `key: 123`
fn find_key_value(input: &str, key: &str) -> Option<String> {
    let key_pos = input.find(key)?;
    let after_key = &input[key_pos + key.len()..];
    let trimmed_after_key = after_key.trim_start();

    if trimmed_after_key.starts_with('"') {
        // Handle quoted string value
        let start_idx = key_pos + key.len() + trimmed_after_key.len()
            - trimmed_after_key.trim_start_matches('"').len();

        // Find the closing quote, handling escaped quotes
        let mut chars = trimmed_after_key[1..].char_indices();
        let mut prev_char = ' ';
        while let Some((idx, ch)) = chars.next() {
            if ch == '"' && prev_char != '\\' {
                // Found the end of the quoted string
                let value = &input[start_idx + 1..start_idx + idx + 1];
                return Some(value.to_string());
            }
            prev_char = ch;
        }
        None
    } else {
        // Handle non-quoted value (likely a number)
        let start_idx = key_pos + key.len();
        let remaining = &input[start_idx..];
        let trimmed_remaining = remaining.trim_start();

        // Find the end of the value (next comma, space+}, or }
        let mut chars = trimmed_remaining.char_indices();
        let mut end_pos = trimmed_remaining.len();
        while let Some((idx, ch)) = chars.next() {
            if ch == ','
                || (ch == ' '
                    && idx + 1 < trimmed_remaining.len()
                    && trimmed_remaining.chars().nth(idx + 1) == Some('}'))
                || ch == '}'
            {
                end_pos = idx;
                break;
            }
        }

        let value = trimmed_remaining[..end_pos].trim();
        if !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StackTrace;

    #[test]
    fn parse_backtrace() {
        let backtrace = std::backtrace::Backtrace::force_capture();

        let msg = format!("{:?}", backtrace);

        let stack = StackTrace::parse_debug_str(&msg);

        assert!(stack.is_some());
        let stack = stack.unwrap();
        println!("len {}", stack.frames.len());

        println!("{}", backtrace);
        println!("{:?}", stack);
    }
}
