use crate::types::{SourceMap, SourceMapEntry};
use std::collections::HashMap;

pub struct SourceMapBuilder {
    entries: Vec<SourceMapEntry>,
    expanded_to_source: HashMap<String, Vec<SourceMapEntry>>,
    source_to_expanded: HashMap<String, Vec<SourceMapEntry>>,
}

impl SourceMapBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            expanded_to_source: HashMap::new(),
            source_to_expanded: HashMap::new(),
        }
    }
    
    pub fn add_mapping(&mut self, entry: SourceMapEntry) {
        // Add to expanded lookup
        let expanded_key = format!("{}:{}", entry.expanded_range.start.line, entry.expanded_range.start.column);
        self.expanded_to_source
            .entry(expanded_key.clone())
            .or_insert_with(Vec::new)
            .push(entry.clone());
        
        // Also index by line for range-based lookups
        let expanded_line_key = format!("line:{}", entry.expanded_range.start.line);
        self.expanded_to_source
            .entry(expanded_line_key)
            .or_insert_with(Vec::new)
            .push(entry.clone());
        
        // Add to source lookup
        let source_key = format!("{}:{}", entry.source_range.start.line, entry.source_range.start.column);
        self.source_to_expanded
            .entry(source_key)
            .or_insert_with(Vec::new)
            .push(entry.clone());
        
        // Also index by line for range-based lookups
        let source_line_key = format!("line:{}", entry.source_range.start.line);
        self.source_to_expanded
            .entry(source_line_key)
            .or_insert_with(Vec::new)
            .push(entry.clone());
        
        self.entries.push(entry);
    }
    
    pub fn build(self) -> SourceMap {
        SourceMap {
            version: 1,
            entries: self.entries,
            expanded_to_source: self.expanded_to_source,
            source_to_expanded: self.source_to_expanded,
        }
    }
}