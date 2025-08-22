use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: Rc<String>,
    pub line: usize,
    pub original_line: usize,
}

#[derive(Debug)]
pub struct PreprocessedLine {
    pub content: String,
    pub location: SourceLocation,
}

pub struct Preprocessor {
    include_paths: Vec<PathBuf>,
    included_files: HashSet<PathBuf>,
    pragma_once_files: HashSet<PathBuf>,
    defined_macros: HashSet<String>,
    include_depth: usize,
    max_include_depth: usize,
    current_file: Option<PathBuf>,
    errors: Vec<String>,
}

impl Preprocessor {
    pub fn new(include_paths: Vec<PathBuf>) -> Self {
        Self {
            include_paths,
            included_files: HashSet::new(),
            pragma_once_files: HashSet::new(),
            defined_macros: HashSet::new(),
            include_depth: 0,
            max_include_depth: 100,
            current_file: None,
            errors: Vec::new(),
        }
    }

    pub fn preprocess(&mut self, input: &str, source_file: Option<&Path>) -> Result<String, Vec<String>> {
        self.included_files.clear();
        self.include_depth = 0;
        self.errors.clear();
        
        if let Some(file) = source_file {
            self.current_file = Some(file.to_path_buf());
            self.included_files.insert(file.to_path_buf());
        }

        let filename = source_file
            .and_then(|p| p.to_str())
            .unwrap_or("<input>")
            .to_string();
        
        let result = self.process_content(input, Rc::new(filename))?;
        
        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        
        // Reconstruct the preprocessed content with line directives
        let mut output = String::new();
        let mut last_file = None;
        let mut last_line = None;
        
        for line in result {
            // Add line directive if file or line number changed significantly
            let needs_directive = if let Some(ref last) = last_file {
                last != &line.location.file || 
                    last_line.map_or(true, |l: usize| line.location.original_line != l + 1)
            } else {
                true
            };
            
            if needs_directive {
                output.push_str(&format!("#line {} \"{}\"\n", 
                    line.location.original_line, 
                    line.location.file));
            }
            
            output.push_str(&line.content);
            output.push('\n');
            
            last_file = Some(line.location.file.clone());
            last_line = Some(line.location.original_line);
        }
        
        Ok(output)
    }

    fn process_content(&mut self, content: &str, source_file: Rc<String>) -> Result<Vec<PreprocessedLine>, Vec<String>> {
        let mut result = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        // Check for #pragma once at the beginning of the file
        let mut has_pragma_once = false;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("#pragma once") {
                has_pragma_once = true;
                break;
            } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
                // Stop checking after first non-comment, non-empty line
                break;
            }
        }
        
        if has_pragma_once {
            if let Some(ref current) = self.current_file {
                if let Ok(canonical) = current.canonicalize() {
                    if self.pragma_once_files.contains(&canonical) {
                        // File already included with pragma once, skip it
                        return Ok(vec![]);
                    }
                    self.pragma_once_files.insert(canonical);
                }
            }
        }
        
        let mut i = 0;
        let mut skip_until_endif = 0;  // Count of nested ifdefs to skip
        
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            
            // Handle preprocessor directives
            if trimmed.starts_with("#pragma once") {
                // Already handled above, skip the line
                i += 1;
                continue;
            } else if trimmed.starts_with("#ifndef") {
                let macro_name = trimmed.strip_prefix("#ifndef").unwrap().trim();
                if self.defined_macros.contains(macro_name) {
                    skip_until_endif += 1;
                }
            } else if trimmed.starts_with("#ifdef") {
                let macro_name = trimmed.strip_prefix("#ifdef").unwrap().trim();
                if !self.defined_macros.contains(macro_name) {
                    skip_until_endif += 1;
                }
            } else if trimmed.starts_with("#define") && skip_until_endif == 0 {
                let define_line = trimmed.strip_prefix("#define").unwrap().trim();
                if let Some(space_pos) = define_line.find(|c: char| c.is_whitespace()) {
                    let macro_name = &define_line[..space_pos];
                    self.defined_macros.insert(macro_name.to_string());
                } else {
                    // #define with just a name
                    self.defined_macros.insert(define_line.to_string());
                }
                // Still include the #define line in output for macro expansion
                result.push(PreprocessedLine {
                    content: line.to_string(),
                    location: SourceLocation {
                        file: source_file.clone(),
                        line: i + 1,
                        original_line: i + 1,
                    },
                });
            } else if trimmed.starts_with("#endif") {
                if skip_until_endif > 0 {
                    skip_until_endif -= 1;
                }
            } else if skip_until_endif > 0 {
                // We're inside a skipped ifdef block, don't process this line
            } else if trimmed.starts_with("#include") {
                let include_line = trimmed.strip_prefix("#include").unwrap().trim();
                
                if let Some(included_lines) = self.process_include(include_line, &source_file, i + 1)? {
                    result.extend(included_lines);
                }
            } else if !trimmed.starts_with("#line") {
                // Skip #line directives in input as we'll regenerate them
                result.push(PreprocessedLine {
                    content: line.to_string(),
                    location: SourceLocation {
                        file: source_file.clone(),
                        line: i + 1,
                        original_line: i + 1,
                    },
                });
            }
            
            i += 1;
        }
        
        Ok(result)
    }

    fn process_include(&mut self, include_line: &str, current_file: &Rc<String>, line_num: usize) -> Result<Option<Vec<PreprocessedLine>>, Vec<String>> {
        self.include_depth += 1;
        
        if self.include_depth > self.max_include_depth {
            self.errors.push(format!(
                "{}:{}: Maximum include depth ({}) exceeded",
                current_file, line_num, self.max_include_depth
            ));
            self.include_depth -= 1;
            return Err(self.errors.clone());
        }
        
        let (path_str, is_system) = if include_line.starts_with('"') && include_line.ends_with('"') {
            // Local include: "file.bfm"
            let path = include_line.trim_matches('"');
            (path, false)
        } else if include_line.starts_with('<') && include_line.ends_with('>') {
            // System include: <file.bfm>
            let path = include_line.trim_matches('<').trim_matches('>');
            (path, true)
        } else {
            self.errors.push(format!(
                "{}:{}: Invalid include syntax: {}",
                current_file, line_num, include_line
            ));
            self.include_depth -= 1;
            return Err(self.errors.clone());
        };
        
        let resolved_path = self.resolve_include_path(path_str, is_system, current_file)?;
        
        // Check for circular includes (but allow re-inclusion with guards)
        let canonical_path = resolved_path.canonicalize().unwrap_or(resolved_path.clone());
        
        // Only check for circular includes in the current include chain, not all ever included
        // This allows files with include guards to be included multiple times from different files
        if self.included_files.contains(&canonical_path) {
            // This is a circular dependency in the current chain
            self.errors.push(format!(
                "{}:{}: Circular include detected: {}",
                current_file, line_num, path_str
            ));
            self.include_depth -= 1;
            return Err(self.errors.clone());
        }
        
        // Read the file
        let content = match fs::read_to_string(&canonical_path) {
            Ok(content) => content,
            Err(e) => {
                self.errors.push(format!(
                    "{}:{}: Failed to read include file '{}': {}",
                    current_file, line_num, path_str, e
                ));
                self.include_depth -= 1;
                return Err(self.errors.clone());
            }
        };
        
        // Mark as included
        self.included_files.insert(canonical_path.clone());
        
        // Process the included content recursively
        let included_filename = canonical_path
            .to_str()
            .unwrap_or(path_str)
            .to_string();
        
        let old_current = self.current_file.clone();
        self.current_file = Some(canonical_path.clone());
        
        let included_lines = self.process_content(&content, Rc::new(included_filename))?;
        
        self.current_file = old_current;
        self.included_files.remove(&canonical_path);
        self.include_depth -= 1;
        
        Ok(Some(included_lines))
    }

    fn resolve_include_path(&self, path_str: &str, is_system: bool, _current_file: &str) -> Result<PathBuf, Vec<String>> {
        let path = Path::new(path_str);
        
        if !is_system {
            // For local includes, first try relative to current file
            if let Some(current_path) = self.current_file.as_ref() {
                if let Some(parent) = current_path.parent() {
                    let candidate = parent.join(path);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
            
            // Then try relative to working directory
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }
        
        // Try include paths
        for include_path in &self.include_paths {
            let candidate = include_path.join(path);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        
        // If not system include, try current directory as fallback
        if !is_system && path.exists() {
            return Ok(path.to_path_buf());
        }
        
        Err(vec![format!(
            "Cannot find include file '{}' (searched {} paths)",
            path_str,
            self.include_paths.len() + if is_system { 0 } else { 2 }
        )])
    }
}

/// Parse #line directives to build a source map
pub fn parse_line_directives(input: &str) -> (String, LineMap) {
    let mut clean_lines = Vec::new();
    let mut line_map = LineMap::new();
    let mut current_file = "<input>".to_string();
    let mut current_source_line = 1;
    let mut output_line = 1;
    
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#line") {
            // Parse: #line 123 "filename"
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(line_num) = parts[1].parse::<usize>() {
                    current_source_line = line_num;
                    if parts.len() > 2 {
                        current_file = parts[2..].join(" ").trim_matches('"').to_string();
                    }
                }
            }
        } else {
            clean_lines.push(line);
            line_map.add_mapping(output_line, current_file.clone(), current_source_line);
            current_source_line += 1;
            output_line += 1;
        }
    }
    
    (clean_lines.join("\n"), line_map)
}

/// Remove #line directives from preprocessed output (for final expansion)
pub fn strip_line_directives(input: &str) -> String {
    input.lines()
        .filter(|line| !line.trim().starts_with("#line"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone)]
pub struct LineMapping {
    pub output_line: usize,
    pub source_file: String,
    pub source_line: usize,
}

pub struct LineMap {
    mappings: Vec<LineMapping>,
}

impl LineMap {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }
    
    pub fn add_mapping(&mut self, output_line: usize, source_file: String, source_line: usize) {
        self.mappings.push(LineMapping {
            output_line,
            source_file,
            source_line,
        });
    }
    
    pub fn get_source_location(&self, output_line: usize) -> Option<(String, usize)> {
        // Find the last mapping that applies to this line
        for mapping in self.mappings.iter().rev() {
            if mapping.output_line <= output_line {
                let offset = output_line - mapping.output_line;
                return Some((
                    mapping.source_file.clone(),
                    mapping.source_line + offset,
                ));
            }
        }
        None
    }
}