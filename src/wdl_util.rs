//! Module for utility functions.
//!
//! Currently that is whitespace stripping.

fn remove_line_continuations(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                if next == '\n' {
                    while let Some(ws) = chars.next() {
                        if ws == ' ' || ws == '\t' {
                            continue;
                        } else {
                            result.push(ws);
                            break;
                        }
                    }
                }
            }
        }
        result.push(c);
    }
    result
}

fn calculate_leading_whitespace(s: &str) -> usize {
    let mut min_leading_whitespace = usize::MAX;
    let mut parsing_leading_whitespace = true;
    let mut cur_char_index = 0;
    let mut cur_line_index = 0;
    s.chars().for_each(|c| match c {
        ' ' | '\t' if parsing_leading_whitespace => cur_char_index += 1,
        '\n' => {
            parsing_leading_whitespace = true;
            cur_line_index += 1;
            cur_char_index = 0;
        }
        _ => {
            parsing_leading_whitespace = false;
            if cur_char_index < min_leading_whitespace {
                min_leading_whitespace = cur_char_index;
            }
            cur_char_index += 1;
        }
    });
    if min_leading_whitespace == usize::MAX {
        0
    } else {
        min_leading_whitespace
    }
}

/// Strips leading whitespace from a string.
pub fn strip_leading_whitespace(s: &str, command: bool) -> String {
    let s_owned = if command {
        s.to_string()
    } else {
        remove_line_continuations(s)
    };
    let leading_whitespace = calculate_leading_whitespace(&s_owned);
    let result = s_owned
        .lines()
        .map(|line| {
            if line.len() >= leading_whitespace {
                &line[leading_whitespace..]
            } else {
                ""
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_line_continuations() {
        let input = "first line \
                     still first line
second line";
        let expected = "first line still first line\nsecond line";
        assert_eq!(remove_line_continuations(input), expected);
    }

    #[test]
    fn test_calculate_leading_whitespace() {
        let input = "    first line is indented 4 spaces
        second line is indented 8 spaces
    third line is indented 4 spaces";
        assert_eq!(calculate_leading_whitespace(input), 4);
        let input = "    first line is indented 4 spaces
 \t \t    second line is indented with a mix of 8 spaces and tabs
\t\t\t\tfourth line is indented 4 tabs";
        assert_eq!(calculate_leading_whitespace(input), 4);
    }

    #[test]
    fn test_strip_leading_whitespace_not_in_command() {
        let input = "    first line is indented 4 spaces \
                     still first line
        second line is indented 8 spaces
    third line is indented 4 spaces";
        let expected = "first line is indented 4 spaces still first line\n    second line is indented 8 spaces\nthird line is indented 4 spaces";
        assert_eq!(strip_leading_whitespace(input, false), expected);
    }

    #[test]
    fn test_strip_leading_whitespace_in_command() {
        let input = "    first line is indented 4 spaces and trails a backslash \\
        second line is indented 8 spaces
    third line is indented 4 spaces";
        let expected = "first line is indented 4 spaces and trails a backslash \\\n    second line is indented 8 spaces\nthird line is indented 4 spaces";
        assert_eq!(strip_leading_whitespace(input, true), expected);
    }
}
