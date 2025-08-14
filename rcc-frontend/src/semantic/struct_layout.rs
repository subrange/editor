//! Struct layout calculation
//! 
//! This module handles computing field offsets and total sizes for structs.
//! IMPORTANT: We explicitly error on unsupported features rather than silently
//! generating incorrect layouts.

use crate::types::{Type, StructField};
use crate::CompilerError;
use rcc_common::SourceLocation;
use std::collections::HashMap;

/// Information about a struct's memory layout
#[derive(Debug, Clone)]
pub struct StructLayout {
    pub fields: Vec<FieldLayout>,
    pub total_size: u64,  // Size in words
}

/// Layout information for a single field
#[derive(Debug, Clone)]
pub struct FieldLayout {
    pub name: String,
    pub field_type: Type,
    pub offset: u64,  // Offset in words from start of struct
    pub size: u64,    // Size in words
}

/// Calculate the memory layout for a struct
/// 
/// This function computes field offsets and total size.
/// Currently implements simple sequential layout without padding/alignment.
/// 
/// # Errors
/// Returns error if:
/// - A field has an incomplete type (e.g., void, unsized array)
/// - A field contains itself (recursive struct)
/// - The struct is too large (overflow)
pub fn calculate_struct_layout(
    fields: &[StructField],
    location: SourceLocation,
) -> Result<StructLayout, CompilerError> {
    let mut layout_fields = Vec::new();
    let mut current_offset = 0u64;
    
    for field in fields {
        // Get the size of this field
        let field_size = match field.field_type.size_in_words() {
            Some(size) => size,
            None => {
                return Err(crate::semantic::SemanticError::IncompleteType {
                    type_name: format!("field '{}' has incomplete type {:?}", 
                                     field.name, field.field_type),
                    location: location.clone(),
                }.into());
            }
        };
        
        // Check for overflow
        let new_offset = current_offset.checked_add(field_size)
            .ok_or_else(|| crate::semantic::SemanticError::StructTooLarge {
                struct_name: "struct".to_string(),
                location: location.clone(),
            })?;
        
        // Add field to layout
        layout_fields.push(FieldLayout {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
            offset: current_offset,
            size: field_size,
        });
        
        current_offset = new_offset;
    }
    
    Ok(StructLayout {
        fields: layout_fields,
        total_size: current_offset,
    })
}

/// Find a field in a struct layout by name
pub fn find_field<'a>(
    layout: &'a StructLayout,
    field_name: &str,
) -> Option<&'a FieldLayout> {
    layout.fields.iter().find(|f| f.name == field_name)
}

/// Check if a type recursively contains itself
/// This is used to detect invalid recursive struct definitions
pub fn check_recursive_struct(
    ty: &Type,
    seen_types: &mut HashMap<String, bool>,
) -> bool {
    match ty {
        Type::Struct { name: Some(name), fields } => {
            // Check if we've seen this type before
            if let Some(&is_recursive) = seen_types.get(name) {
                return is_recursive;
            }
            
            // Mark as being checked (to handle mutual recursion)
            seen_types.insert(name.clone(), false);
            
            // Check each field
            for field in fields {
                // Direct containment (not via pointer) is recursive
                if !field.field_type.is_pointer() {
                    if let Type::Struct { name: Some(field_name), .. } = &field.field_type {
                        if field_name == name {
                            seen_types.insert(name.clone(), true);
                            return true;
                        }
                    }
                    
                    // Check nested structs
                    if check_recursive_struct(&field.field_type, seen_types) {
                        seen_types.insert(name.clone(), true);
                        return true;
                    }
                }
            }
            
            seen_types.insert(name.clone(), false);
            false
        }
        Type::Array { element_type, .. } => {
            // Arrays directly contain their element type
            check_recursive_struct(element_type, seen_types)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_struct_layout() {
        let fields = vec![
            StructField {
                name: "x".to_string(),
                field_type: Type::Int,
                offset: None,
            },
            StructField {
                name: "y".to_string(),
                field_type: Type::Int,
                offset: None,
            },
            StructField {
                name: "z".to_string(),
                field_type: Type::Long,  // 2 words
                offset: None,
            },
        ];
        
        let layout = calculate_struct_layout(&fields, SourceLocation::new_simple(0, 0))
            .expect("Should calculate layout");
        
        assert_eq!(layout.total_size, 4);  // 1 + 1 + 2 words
        assert_eq!(layout.fields.len(), 3);
        
        assert_eq!(layout.fields[0].name, "x");
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[0].size, 1);
        
        assert_eq!(layout.fields[1].name, "y");
        assert_eq!(layout.fields[1].offset, 1);
        assert_eq!(layout.fields[1].size, 1);
        
        assert_eq!(layout.fields[2].name, "z");
        assert_eq!(layout.fields[2].offset, 2);
        assert_eq!(layout.fields[2].size, 2);
    }
    
    #[test]
    fn test_struct_with_arrays() {
        let fields = vec![
            StructField {
                name: "arr".to_string(),
                field_type: Type::Array {
                    element_type: Box::new(Type::Int),
                    size: Some(10),
                },
                offset: None,
            },
            StructField {
                name: "x".to_string(),
                field_type: Type::Int,
                offset: None,
            },
        ];
        
        let layout = calculate_struct_layout(&fields, SourceLocation::new_simple(0, 0))
            .expect("Should calculate layout");
        
        assert_eq!(layout.total_size, 11);  // 10 + 1 words
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[0].size, 10);
        assert_eq!(layout.fields[1].offset, 10);
        assert_eq!(layout.fields[1].size, 1);
    }
    
    #[test]
    fn test_struct_with_pointers() {
        let fields = vec![
            StructField {
                name: "ptr".to_string(),
                field_type: Type::Pointer {
                    target: Box::new(Type::Int),
                    bank: None,
                },
                offset: None,
            },
            StructField {
                name: "x".to_string(),
                field_type: Type::Int,
                offset: None,
            },
        ];
        
        let layout = calculate_struct_layout(&fields, SourceLocation::new_simple(0, 0))
            .expect("Should calculate layout");
        
        assert_eq!(layout.total_size, 3);  // 2 (fat ptr) + 1 words
        assert_eq!(layout.fields[0].size, 2);  // Fat pointer is 2 words
    }
    
    #[test]
    fn test_nested_structs() {
        let inner_struct = Type::Struct {
            name: Some("Inner".to_string()),
            fields: vec![
                StructField {
                    name: "a".to_string(),
                    field_type: Type::Int,
                    offset: None,
                },
                StructField {
                    name: "b".to_string(),
                    field_type: Type::Int,
                    offset: None,
                },
            ],
        };
        
        let fields = vec![
            StructField {
                name: "inner".to_string(),
                field_type: inner_struct,
                offset: None,
            },
            StructField {
                name: "x".to_string(),
                field_type: Type::Int,
                offset: None,
            },
        ];
        
        let layout = calculate_struct_layout(&fields, SourceLocation::new_simple(0, 0))
            .expect("Should calculate layout");
        
        assert_eq!(layout.total_size, 3);  // 2 (inner) + 1
        assert_eq!(layout.fields[0].size, 2);
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[1].offset, 2);
    }
    
    #[test]
    fn test_empty_struct() {
        let fields = vec![];
        
        let layout = calculate_struct_layout(&fields, SourceLocation::new_simple(0, 0))
            .expect("Should calculate layout");
        
        assert_eq!(layout.total_size, 0);
        assert_eq!(layout.fields.len(), 0);
    }
    
    #[test]
    fn test_incomplete_type_error() {
        let fields = vec![
            StructField {
                name: "bad".to_string(),
                field_type: Type::Void,  // Void has no size
                offset: None,
            },
        ];
        
        let result = calculate_struct_layout(&fields, SourceLocation::new_simple(0, 0));
        assert!(result.is_err());
        
        if let Err(err) = result {
            let err_str = format!("{}", err);
            assert!(err_str.contains("incomplete type"));
        }
    }
    
    #[test]
    fn test_unsized_array_error() {
        let fields = vec![
            StructField {
                name: "flex".to_string(),
                field_type: Type::Array {
                    element_type: Box::new(Type::Int),
                    size: None,  // Unsized array
                },
                offset: None,
            },
        ];
        
        let result = calculate_struct_layout(&fields, SourceLocation::new_simple(0, 0));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_find_field() {
        let fields = vec![
            StructField {
                name: "x".to_string(),
                field_type: Type::Int,
                offset: None,
            },
            StructField {
                name: "y".to_string(),
                field_type: Type::Int,
                offset: None,
            },
        ];
        
        let layout = calculate_struct_layout(&fields, SourceLocation::new_simple(0, 0))
            .expect("Should calculate layout");
        
        let field = find_field(&layout, "y").expect("Should find field y");
        assert_eq!(field.name, "y");
        assert_eq!(field.offset, 1);
        
        assert!(find_field(&layout, "z").is_none());
    }
    
    #[test]
    fn test_recursive_struct_detection() {
        let mut seen = HashMap::new();
        
        // Direct recursion - struct contains itself
        let recursive = Type::Struct {
            name: Some("Node".to_string()),
            fields: vec![
                StructField {
                    name: "data".to_string(),
                    field_type: Type::Int,
                    offset: None,
                },
                StructField {
                    name: "next".to_string(),
                    field_type: Type::Struct {
                        name: Some("Node".to_string()),
                        fields: vec![],  // Would be filled in reality
                    },
                    offset: None,
                },
            ],
        };
        
        assert!(check_recursive_struct(&recursive, &mut seen));
        
        // Pointer breaks recursion - this is valid
        seen.clear();
        let valid = Type::Struct {
            name: Some("Node".to_string()),
            fields: vec![
                StructField {
                    name: "data".to_string(),
                    field_type: Type::Int,
                    offset: None,
                },
                StructField {
                    name: "next".to_string(),
                    field_type: Type::Pointer {
                        target: Box::new(Type::Struct {
                            name: Some("Node".to_string()),
                            fields: vec![],
                        }),
                        bank: None,
                    },
                    offset: None,
                },
            ],
        };
        
        assert!(!check_recursive_struct(&valid, &mut seen));
    }
}