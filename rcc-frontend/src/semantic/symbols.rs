//! Symbol resolution and management
//! 
//! This module handles symbol table operations and function/global declarations.

use crate::ast::*;
use crate::semantic::errors::SemanticError;
use crate::semantic::types::TypeAnalyzer;
use rcc_common::{CompilerError, SymbolTable, SymbolId, SourceLocation, StorageClass as CommonStorageClass};
use std::collections::HashMap;
use crate::{StorageClass, Type};

pub struct SymbolManager<'a> {
    pub symbol_table: &'a mut SymbolTable,
    pub symbol_locations: &'a mut HashMap<SymbolId, SourceLocation>,
    pub symbol_types: &'a mut HashMap<SymbolId, Type>,
    pub type_definitions: &'a mut HashMap<String, Type>,
}

impl<'a> SymbolManager<'a> {
    /// Declare a function in the symbol table
    pub fn declare_function(&mut self, func: &FunctionDefinition) -> Result<(), CompilerError> {
        if self.symbol_table.exists_in_current_scope(&func.name) {
            return Err(SemanticError::RedefinedSymbol {
                name: func.name.clone(),
                original_location: SourceLocation::new_simple(0, 0), // TODO: Track original location
                redefinition_location: func.span.start.clone(),
            }.into());
        }
        
        let func_type = Type::Function {
            return_type: Box::new(func.return_type.clone()),
            parameters: func.parameters.iter().map(|p| p.param_type.clone()).collect(),
            is_variadic: false, // TODO: Handle variadic functions
        };
        
        let symbol_id = self.symbol_table.add_symbol(func.name.clone());
        self.symbol_locations.insert(symbol_id, func.span.start.clone());
        // Store the function type
        self.symbol_types.insert(symbol_id, func_type);
        
        if let Some(symbol) = self.symbol_table.get_symbol_mut(symbol_id) {
            *symbol = symbol.clone()
                .as_function()
                .as_defined()
                .with_storage_class(
                    match func.storage_class {
                        StorageClass::Static => CommonStorageClass::Static,
                        StorageClass::Extern => CommonStorageClass::Extern,
                        _ => CommonStorageClass::Auto,
                    }
                );
        }
        
        Ok(())
    }
    
    /// Declare a global variable in the symbol table
    pub fn declare_global_variable(&mut self, decl: &mut Declaration) -> Result<(), CompilerError> {
        // Handle typedef specially - it defines a type alias, not a variable
        if decl.storage_class == StorageClass::Typedef {
            // Resolve the type first
            let analyzer = TypeAnalyzer::new(self.type_definitions);
            decl.decl_type = analyzer.resolve_type(&decl.decl_type);
            // Register this as a type definition
            self.type_definitions.insert(decl.name.clone(), decl.decl_type.clone());
            return Ok(());
        }
        
        if self.symbol_table.exists_in_current_scope(&decl.name) {
            return Err(SemanticError::RedefinedSymbol {
                name: decl.name.clone(),
                original_location: SourceLocation::new_simple(0, 0), // TODO: Track original location
                redefinition_location: decl.span.start.clone(),
            }.into());
        }
        
        // Resolve the type (in case it references a named struct/union/enum)
        let analyzer = TypeAnalyzer::new(self.type_definitions);
        decl.decl_type = analyzer.resolve_type(&decl.decl_type);
        
        // Fix storage class for global variables
        // If storage class is Auto (the default), change it to Extern for globals
        if decl.storage_class == StorageClass::Auto {
            decl.storage_class = StorageClass::Extern;
        }
        
        let symbol_id = self.symbol_table.add_symbol(decl.name.clone());
        self.symbol_locations.insert(symbol_id, decl.span.start.clone());
        // Store the global variable type
        self.symbol_types.insert(symbol_id, decl.decl_type.clone());
        
        if let Some(symbol) = self.symbol_table.get_symbol_mut(symbol_id) {
            *symbol = symbol.clone()
                .with_storage_class(
                    match decl.storage_class {
                        StorageClass::Static => CommonStorageClass::Static,
                        StorageClass::Extern => CommonStorageClass::Extern,
                        StorageClass::Register => CommonStorageClass::Register,
                        _ => CommonStorageClass::Extern,  // Default to Extern for globals
                    }
                );
        }
        
        Ok(())
    }
    
    /// Register a type definition (struct, union, enum)
    pub fn register_type_definition(&mut self, name: String, mut type_def: Type) -> Result<(), CompilerError> {
        // Check if type already exists
        if self.type_definitions.contains_key(&name) {
            return Err(SemanticError::RedefinedType {
                name: name.clone(),
            }.into());
        }
        
        // Resolve field types in struct/union definitions
        let analyzer = TypeAnalyzer::new(self.type_definitions);
        match &mut type_def {
            Type::Struct { fields, .. } => {
                for field in fields.iter_mut() {
                    field.field_type = analyzer.resolve_type(&field.field_type);
                }
            }
            Type::Union { fields, .. } => {
                for field in fields.iter_mut() {
                    field.field_type = analyzer.resolve_type(&field.field_type);
                }
            }
            _ => {}
        }
        
        // Store the type definition with resolved field types
        self.type_definitions.insert(name, type_def);
        
        Ok(())
    }
    
    /// Add function parameters to the symbol table
    pub fn add_function_parameters(&mut self, parameters: &mut [Parameter]) -> Result<(), CompilerError> {
        for param in parameters {
            if let Some(name) = &param.name {
                let symbol_id = self.symbol_table.add_symbol(name.clone());
                param.symbol_id = Some(symbol_id);
                self.symbol_locations.insert(symbol_id, param.span.start.clone());
                // Store the parameter type
                self.symbol_types.insert(symbol_id, param.param_type.clone());
            }
        }
        Ok(())
    }
}