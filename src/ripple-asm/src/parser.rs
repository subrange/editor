use crate::types::{ParsedLine, Section};

pub struct Parser {
    case_insensitive: bool,
}

impl Parser {
    pub fn new(case_insensitive: bool) -> Self {
        Self { case_insensitive }
    }

    pub fn parse_source(&self, source: &str) -> Vec<ParsedLine> {
        let lines: Vec<&str> = source.lines().collect();
        let mut parsed = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let parsed_line = self.parse_line(line, i + 1);
            if parsed_line.label.is_some() 
                || parsed_line.mnemonic.is_some() 
                || parsed_line.directive.is_some() {
                parsed.push(parsed_line);
            }
        }

        parsed
    }

    fn parse_line(&self, line: &str, line_number: usize) -> ParsedLine {
        let raw = line.to_string();
        
        // Remove comments
        let line = if let Some(pos) = line.find(';') {
            &line[..pos]
        } else if let Some(pos) = line.find('#') {
            &line[..pos]
        } else if let Some(pos) = line.find("//") {
            &line[..pos]
        } else {
            line
        };

        let line = line.trim();

        let mut result = ParsedLine {
            label: None,
            mnemonic: None,
            operands: Vec::new(),
            directive: None,
            directive_args: Vec::new(),
            line_number,
            raw,
        };

        if line.is_empty() {
            return result;
        }

        // Check for directives first (start with .)
        if line.starts_with('.') {
            let tokens = self.tokenize(line);
            if !tokens.is_empty() {
                result.directive = Some(tokens[0][1..].to_lowercase());
                result.directive_args = tokens[1..].to_vec();
            }
            return result;
        }

        // Check for labels
        let (label, rest) = if let Some(colon_pos) = line.find(':') {
            let label = line[..colon_pos].trim().to_string();
            let rest = line[colon_pos + 1..].trim();
            (Some(label), rest)
        } else {
            (None, line)
        };

        result.label = label;

        if rest.is_empty() {
            return result;
        }

        // Check for directives after label
        if rest.starts_with('.') {
            let tokens = self.tokenize(rest);
            if !tokens.is_empty() {
                result.directive = Some(tokens[0][1..].to_lowercase());
                result.directive_args = tokens[1..].to_vec();
            }
            return result;
        }

        // Parse instruction
        let tokens = self.tokenize(rest);
        if !tokens.is_empty() {
            result.mnemonic = Some(if self.case_insensitive {
                tokens[0].to_uppercase()
            } else {
                tokens[0].to_string()
            });
            result.operands = tokens[1..].to_vec();
        }

        result
    }

    fn tokenize(&self, line: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut string_char = '\0';
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if in_string {
                if ch == string_char {
                    if chars.peek() == Some(&string_char) {
                        // Escaped quote
                        current.push(ch);
                        chars.next();
                    } else {
                        // End of string
                        in_string = false;
                        tokens.push(format!("{}{}{}", string_char, current, string_char));
                        current.clear();
                    }
                } else if ch == '\\' && chars.peek().is_some() {
                    // Escape sequence
                    current.push(ch);
                    if let Some(next) = chars.next() {
                        current.push(next);
                    }
                } else {
                    current.push(ch);
                }
            } else {
                match ch {
                    ' ' | '\t' => {
                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                    }
                    ',' => {
                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                    }
                    '"' | '\'' => {
                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                        in_string = true;
                        string_char = ch;
                    }
                    _ => {
                        current.push(ch);
                    }
                }
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_instruction() {
        let parser = Parser::new(true);
        let lines = parser.parse_source("ADD R3, R4, R5");
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].mnemonic.as_ref().unwrap(), "ADD");
        assert_eq!(lines[0].operands, vec!["R3", "R4", "R5"]);
    }

    #[test]
    fn test_parse_label() {
        let parser = Parser::new(true);
        let lines = parser.parse_source("loop: ADD R3, R4, R5");
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].label.as_ref().unwrap(), "loop");
        assert_eq!(lines[0].mnemonic.as_ref().unwrap(), "ADD");
    }

    #[test]
    fn test_parse_directive() {
        let parser = Parser::new(true);
        let lines = parser.parse_source(".data");
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].directive.as_ref().unwrap(), "data");
    }

    #[test]
    fn test_parse_string() {
        let parser = Parser::new(true);
        let lines = parser.parse_source(".asciiz \"Hello, World!\"");
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].directive.as_ref().unwrap(), "asciiz");
        assert_eq!(lines[0].directive_args, vec!["\"Hello, World!\""]);
    }

    #[test]
    fn test_ignore_comments() {
        let parser = Parser::new(true);
        let lines = parser.parse_source("ADD R3, R4, R5 ; This is a comment");
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].mnemonic.as_ref().unwrap(), "ADD");
        assert_eq!(lines[0].operands, vec!["R3", "R4", "R5"]);
    }
}