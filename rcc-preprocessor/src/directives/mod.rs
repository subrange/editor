use crate::{Macro, Preprocessor};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::fs;
use std::path::{PathBuf};

impl Preprocessor {
    /// Handle include directive implementation
    pub fn handle_include_impl(&mut self, path: String, is_system: bool) -> Result<String> {
        // Check include depth
        if self.include_depth >= crate::MAX_INCLUDE_DEPTH {
            return Err(anyhow!("Maximum include depth ({}) exceeded", crate::MAX_INCLUDE_DEPTH));
        }
        
        let file_path = self.find_include_file(&path, is_system)?;
        
        // Canonicalize the path for consistent comparison
        let canonical_path = file_path.canonicalize().unwrap_or(file_path.clone());
        
        // Check if file was marked with #pragma once
        if self.pragma_once_files.contains(&canonical_path) {
            return Ok(String::new()); // Skip this file
        }
        
        // Read and process the included file
        let content = fs::read_to_string(&file_path)
            .map_err(|e| anyhow!("Failed to read include file '{}': {}", file_path.display(), e))?;
        
        // Save current state
        let saved_file = self.current_file.clone();
        let saved_depth = self.include_depth;
        
        self.current_file = Some(canonical_path.clone());
        self.include_depth += 1;
        
        // Process the included file
        let result = self.process(&content, canonical_path);
        
        // Restore state
        self.current_file = saved_file;
        self.include_depth = saved_depth;
        
        result
    }
    
    /// Find include file in search paths
    fn find_include_file(&self, path: &str, is_system: bool) -> Result<PathBuf> {
        if is_system {
            // Search in system include directories
            for dir in &self.include_dirs {
                let full_path = dir.join(path);
                if full_path.exists() {
                    return Ok(full_path);
                }
            }
            Err(anyhow!("Cannot find system include file: {}", path))
        } else {
            // First try relative to current file
            if let Some(current) = &self.current_file {
                if let Some(parent) = current.parent() {
                    let relative_path = parent.join(path);
                    if relative_path.exists() {
                        return Ok(relative_path);
                    }
                }
            }
            
            // Then search in include directories
            for dir in &self.include_dirs {
                let full_path = dir.join(path);
                if full_path.exists() {
                    return Ok(full_path);
                }
            }
            
            // Finally try as absolute or relative to CWD
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                Ok(path_buf)
            } else {
                Err(anyhow!("Cannot find include file: {}", path))
            }
        }
    }
    
    /// Handle define directive implementation
    pub fn handle_define_impl(&mut self, name: String, params: Option<Vec<String>>, body: String, is_variadic: bool) -> Result<()> {
        self.macros.insert(
            name.clone(),
            Macro {
                name,
                params,
                body,
                is_variadic,
            },
        );
        Ok(())
    }
    
    /// Handle undef directive implementation
    pub fn handle_undef_impl(&mut self, name: String) -> Result<()> {
        self.macros.remove(&name);
        Ok(())
    }
    
    /// Evaluate conditional expression
    pub fn evaluate_condition(&self, condition: &str) -> Result<bool> {
        // Simple evaluation for now
        // TODO: Implement proper expression evaluation
        
        let condition = condition.trim();
        
        // Handle defined() operator
        if condition.starts_with("defined(") && condition.ends_with(')') {
            let name = condition[8..condition.len()-1].trim();
            return Ok(self.macros.contains_key(name));
        }
        
        if condition.starts_with("defined ") {
            let name = condition[8..].trim();
            return Ok(self.macros.contains_key(name));
        }
        
        // Handle simple numeric comparisons
        if condition == "0" {
            return Ok(false);
        }
        
        if condition == "1" {
            return Ok(true);
        }
        
        // Check if it's a macro name
        if let Some(macro_def) = self.macros.get(condition) {
            return self.evaluate_condition(&macro_def.body);
        }
        
        // Default to false for now
        Ok(false)
    }
    
    /// Expand macros in text implementation
    pub fn expand_macros_impl(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();
        let mut expanded = true;
        let mut depth = 0;
        const MAX_DEPTH: usize = 100;
        
        while expanded && depth < MAX_DEPTH {
            expanded = false;
            depth += 1;
            
            for (name, macro_def) in &self.macros {
                if macro_def.params.is_none() {
                    // Object-like macro
                    let pattern = format!(r"\b{}\b", regex::escape(name));
                    let re = Regex::new(&pattern)?;
                    
                    if re.is_match(&result) {
                        result = re.replace_all(&result, &macro_def.body).to_string();
                        expanded = true;
                    }
                } else {
                    // Function-like macro
                    result = self.expand_function_macro(&result, name, macro_def)?;
                    // TODO: Track if expansion occurred
                }
            }
        }
        
        if depth >= MAX_DEPTH {
            return Err(anyhow!("Maximum macro expansion depth exceeded"));
        }
        
        Ok(result)
    }
    
    /// Expand function-like macro
    fn expand_function_macro(&self, text: &str, name: &str, macro_def: &Macro) -> Result<String> {
        let params = macro_def.params.as_ref().unwrap();
        
        // Create regex pattern for function-like macro invocation
        let pattern = format!(r"\b{}\s*\(", regex::escape(name));
        let re = Regex::new(&pattern)?;
        
        let mut result = String::new();
        let mut last_end = 0;
        
        for mat in re.find_iter(text) {
            result.push_str(&text[last_end..mat.start()]);
            
            // Parse arguments
            let args_start = mat.end();
            let (args, args_end) = self.parse_macro_arguments(&text[args_start..])?;
            
            if args.len() != params.len() && !macro_def.is_variadic {
                return Err(anyhow!(
                    "Macro '{}' expects {} arguments, got {}",
                    name,
                    params.len(),
                    args.len()
                ));
            }
            
            // Perform substitution
            let mut expanded = macro_def.body.clone();
            for (i, param) in params.iter().enumerate() {
                if i < args.len() {
                    let arg_pattern = format!(r"\b{}\b", regex::escape(param));
                    let arg_re = Regex::new(&arg_pattern)?;
                    expanded = arg_re.replace_all(&expanded, &args[i]).to_string();
                }
            }
            
            // Handle variadic arguments
            if macro_def.is_variadic && args.len() > params.len() {
                let va_args = args[params.len()..].join(", ");
                expanded = expanded.replace("__VA_ARGS__", &va_args);
            }
            
            result.push_str(&expanded);
            last_end = args_start + args_end + 1; // +1 for closing paren
        }
        
        result.push_str(&text[last_end..]);
        Ok(result)
    }
    
    /// Parse macro arguments from text
    fn parse_macro_arguments(&self, text: &str) -> Result<(Vec<String>, usize)> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut in_char = false;
        let mut escape = false;
        let mut pos = 0;
        
        for (i, ch) in text.char_indices() {
            pos = i;
            
            if escape {
                current_arg.push(ch);
                escape = false;
                continue;
            }
            
            match ch {
                '\\' if in_string || in_char => {
                    current_arg.push(ch);
                    escape = true;
                }
                '"' if !in_char => {
                    current_arg.push(ch);
                    in_string = !in_string;
                }
                '\'' if !in_string => {
                    current_arg.push(ch);
                    in_char = !in_char;
                }
                '(' if !in_string && !in_char => {
                    paren_depth += 1;
                    current_arg.push(ch);
                }
                ')' if !in_string && !in_char => {
                    if paren_depth == 0 {
                        // End of arguments
                        if !current_arg.trim().is_empty() {
                            args.push(current_arg.trim().to_string());
                        }
                        return Ok((args, pos));
                    }
                    paren_depth -= 1;
                    current_arg.push(ch);
                }
                ',' if !in_string && !in_char && paren_depth == 0 => {
                    // Argument separator
                    args.push(current_arg.trim().to_string());
                    current_arg.clear();
                }
                _ => {
                    current_arg.push(ch);
                }
            }
        }
        
        Err(anyhow!("Unterminated macro arguments"))
    }
}