//! Expression code generation

use std::collections::HashMap;
use rcc_ir::{Value, IrType, IrBinaryOp, IrBuilder, GlobalVariable, Linkage};
use rcc_common::SourceLocation;
use crate::ast::{Expression, ExpressionKind, BinaryOp, UnaryOp, Type};
use crate::CompilerError;
use super::errors::CodegenError;
use super::types::convert_type;

/// Expression generator context
pub struct ExpressionGenerator<'a> {
    pub builder: &'a mut IrBuilder,
    pub module: &'a mut rcc_ir::Module,
    pub variables: &'a HashMap<String, (Value, IrType)>,
    pub array_variables: &'a std::collections::HashSet<String>,
    pub parameter_variables: &'a std::collections::HashSet<String>,
    pub string_literals: &'a mut HashMap<String, String>,
    pub next_string_id: &'a mut u32,
}

impl<'a> ExpressionGenerator<'a> {
    /// Generate IR for an expression
    pub fn generate(&mut self, expr: &Expression) -> Result<Value, CompilerError> {
        match &expr.kind {
            ExpressionKind::IntLiteral(value) => {
                Ok(Value::Constant(*value))
            }
            
            ExpressionKind::CharLiteral(value) => {
                Ok(Value::Constant(*value as i64))
            }
            
            ExpressionKind::StringLiteral(s) => {
                self.generate_string_literal(s)
            }
            
            ExpressionKind::Identifier { name, .. } => {
                self.generate_identifier(name, expr)
            }
            
            ExpressionKind::Binary { op, left, right } => {
                self.generate_binary_operation(*op, left, right)
            }
            
            ExpressionKind::Unary { op, operand } => {
                self.generate_unary_operation(*op, operand)
            }
            
            ExpressionKind::Call { function, arguments } => {
                self.generate_function_call(function, arguments)
            }
            
            ExpressionKind::SizeofExpr(operand) => {
                // sizeof(expression) - get size of expression's type
                if let Some(ref expr_type) = operand.expr_type {
                    let size = super::types::get_ast_type_size(expr_type);
                    Ok(Value::Constant(size as i64))
                } else {
                    // Fallback if no type info
                    Ok(Value::Constant(2))
                }
            }
            
            ExpressionKind::SizeofType(ast_type) => {
                // sizeof(type) - get size of the type
                let size = super::types::get_ast_type_size(ast_type);
                Ok(Value::Constant(size as i64))
            }
            
            ExpressionKind::Member { object, member, is_pointer } => {
                self.generate_member_access(object, member, *is_pointer)
            }
            
            // TODO: Implement other expression types
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: format!("expression type: {:?}", expr.kind),
                location: expr.span.start.clone(),
            }.into()),
        }
    }
    
    fn generate_string_literal(&mut self, s: &str) -> Result<Value, CompilerError> {
        // Create a unique name for this string literal
        let string_id = *self.next_string_id;
        *self.next_string_id += 1;
        
        // Encode the string bytes in the variable name
        let encoded_name = format!("__str_{}_{}", string_id, 
            s.bytes().map(|b| format!("{:02x}", b)).collect::<String>());
        
        let global = GlobalVariable {
            name: encoded_name.clone(),
            var_type: IrType::Array {
                element_type: Box::new(IrType::I8),
                size: (s.len() + 1) as u64, // +1 for null terminator
            },
            is_constant: true,
            initializer: None, // String data is encoded in the name
            linkage: Linkage::Internal,
            symbol_id: None,
        };
        
        // Add to module
        self.module.add_global(global);
        
        // Store the string data for later
        self.string_literals.insert(encoded_name.clone(), s.to_string());
        
        // Return a pointer to the string
        Ok(Value::Global(encoded_name))
    }
    
    fn generate_identifier(&mut self, name: &str, expr: &Expression) -> Result<Value, CompilerError> {
        if let Some((value, var_type)) = self.variables.get(name) {
            // Check if this is a global variable (needs to be loaded)
            if let Value::Global(_) = value {
                // Global variables always need to be loaded
                let temp = self.builder.build_load(value.clone(), var_type.clone())?;
                Ok(Value::Temp(temp))
            } else {
                // Check if this is a pointer type (variable that needs to be loaded)
                match var_type {
                    IrType::Ptr(element_type) => {
                        // Check if this is an array variable
                        // Arrays decay to pointers when used as rvalues
                        if self.array_variables.contains(name) {
                            // Arrays decay to pointers when used as rvalues
                            Ok(value.clone())
                        } else if self.parameter_variables.contains(name) {
                            // Non-pointer parameters are immutable and used directly
                            // (pointer parameters are now handled with allocas and not in this set)
                            Ok(value.clone())
                        } else if element_type.is_pointer() {
                            // This is a pointer to a pointer (like an alloca of a pointer parameter)
                            // Load the pointer value
                            let temp = self.builder.build_load(value.clone(), *element_type.clone())?;
                            Ok(Value::Temp(temp))
                        } else {
                            // Regular pointer variable (local or global), load its value
                            let temp = self.builder.build_load(value.clone(), *element_type.clone())?;
                            Ok(Value::Temp(temp))
                        }
                    }
                    _ => {
                        // This is a direct value (like function parameters), use as is
                        Ok(value.clone())
                    }
                }
            }
        } else {
            Err(CodegenError::UndefinedVariable {
                name: name.to_string(),
                location: expr.span.start.clone(),
            }.into())
        }
    }
    
    /// Generate binary operation
    pub fn generate_binary_operation(&mut self, op: BinaryOp, left: &Expression, right: &Expression) 
        -> Result<Value, CompilerError> {
        match op {
            BinaryOp::Index => {
                // Array indexing: arr[idx]
                let base_ptr = self.generate(left)?;
                let index = self.generate(right)?;
                
                // Determine the element type from the array/pointer type
                let element_type = if let Some(expr_type) = &left.expr_type {
                    match expr_type {
                        Type::Array { element_type, .. } => {
                            convert_type(element_type, left.span.start.clone())?
                        }
                        Type::Pointer(target) => {
                            convert_type(target, left.span.start.clone())?
                        }
                        _ => IrType::I16 // Default to i16 for int
                    }
                } else {
                    IrType::I16 // Default to i16 for int
                };
                
                // Use GetElementPtr for proper array indexing
                let addr = self.builder.build_pointer_offset(
                    base_ptr,
                    index,
                    IrType::Ptr(Box::new(element_type.clone()))
                )?;
                
                // Load from the calculated address
                let result = self.builder.build_load(addr, element_type)?;
                Ok(Value::Temp(result))
            }
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                // For now, compile as simple arithmetic operations
                // TODO: Implement proper short-circuit evaluation
                let left_val = self.generate(left)?;
                let right_val = self.generate(right)?;
                
                // Both values should be 0 or 1 from comparison operations
                // For AND: both must be true (multiply works)
                // For OR: at least one must be true (add then clamp to 0/1)
                let result_type = IrType::I16;
                
                if op == BinaryOp::LogicalAnd {
                    // AND: result = left & right (both are 0 or 1)
                    let temp = self.builder.build_binary(IrBinaryOp::And, left_val, right_val, result_type.clone())?;
                    Ok(Value::Temp(temp))
                } else {
                    // OR: result = left | right (both are already 0 or 1)
                    let temp = self.builder.build_binary(IrBinaryOp::Or, left_val, right_val, result_type)?;
                    Ok(Value::Temp(temp))
                }
            }
            _ => {
                // Check for pointer arithmetic (pointer + integer or integer + pointer)
                let left_is_ptr = left.expr_type.as_ref().map_or(false, |t| t.is_pointer());
                let right_is_ptr = right.expr_type.as_ref().map_or(false, |t| t.is_pointer());
                
                if (left_is_ptr || right_is_ptr) && (op == BinaryOp::Add || op == BinaryOp::Sub) {
                    // Handle pointer arithmetic
                    if left_is_ptr && !right_is_ptr {
                        // pointer + integer or pointer - integer
                        let ptr_val = self.generate(left)?;
                        let offset_val = self.generate(right)?;
                        
                        // For subtraction, negate the offset
                        let final_offset = if op == BinaryOp::Sub {
                            let neg_temp = self.builder.build_binary(
                                IrBinaryOp::Sub, 
                                Value::Constant(0), 
                                offset_val, 
                                IrType::I16
                            )?;
                            Value::Temp(neg_temp)
                        } else {
                            offset_val
                        };
                        
                        // Use pointer offset which preserves bank information
                        let result = self.builder.build_pointer_offset(
                            ptr_val,
                            final_offset,
                            IrType::Ptr(Box::new(IrType::I16))
                        )?;
                        Ok(result)
                    } else if !left_is_ptr && right_is_ptr && op == BinaryOp::Add {
                        // integer + pointer (commutative)
                        let offset_val = self.generate(left)?;
                        let ptr_val = self.generate(right)?;
                        
                        let result = self.builder.build_pointer_offset(
                            ptr_val,
                            offset_val,
                            IrType::Ptr(Box::new(IrType::I16))
                        )?;
                        Ok(result)
                    } else if left_is_ptr && right_is_ptr && op == BinaryOp::Sub {
                        // pointer - pointer (returns integer difference)
                        // This is more complex and not needed for the current test
                        return Err(CodegenError::UnsupportedConstruct {
                            construct: "pointer difference".to_string(),
                            location: left.span.start.clone(),
                        }.into());
                    } else {
                        return Err(CodegenError::UnsupportedConstruct {
                            construct: "invalid pointer arithmetic".to_string(),
                            location: left.span.start.clone(),
                        }.into());
                    }
                } else {
                    // Regular binary operation
                    let left_val = self.generate(left)?;
                    let right_val = self.generate(right)?;
                    
                    let ir_op = convert_binary_op(op).map_err(|msg| {
                        CodegenError::UnsupportedConstruct {
                            construct: msg,
                            location: left.span.start.clone(),
                        }
                    })?;
                    let result_type = IrType::I16; // Simplified for MVP
                    
                    let temp = self.builder.build_binary(ir_op, left_val, right_val, result_type)?;
                    Ok(Value::Temp(temp))
                }
            }
        }
    }
    
    /// Generate unary operation
    pub fn generate_unary_operation(&mut self, op: UnaryOp, operand: &Expression) 
        -> Result<Value, CompilerError> {
        match op {
            UnaryOp::AddressOf => {
                // &expr - get address of expression
                self.generate_lvalue(operand)
            }
            UnaryOp::Dereference => {
                // *expr - dereference pointer
                let ptr = self.generate(operand)?;
                
                // Determine the result type from the expression's type
                let result_type = if let Some(expr_type) = &operand.expr_type {
                    if let Some(target_type) = expr_type.pointer_target() {
                        convert_type(target_type, operand.span.start.clone())?
                    } else {
                        IrType::I8 // Default to byte if we can't determine
                    }
                } else {
                    IrType::I8 // Default to byte
                };
                
                let temp = self.builder.build_load(ptr, result_type)?;
                Ok(Value::Temp(temp))
            }
            UnaryOp::Minus => {
                // Generate 0 - operand
                let operand_val = self.generate(operand)?;
                let zero = Value::Constant(0);
                let result_type = IrType::I16; // Simplified for MVP
                let temp = self.builder.build_binary(IrBinaryOp::Sub, zero, operand_val, result_type)?;
                Ok(Value::Temp(temp))
            }
            UnaryOp::LogicalNot => {
                // Generate operand == 0
                let operand_val = self.generate(operand)?;
                let zero = Value::Constant(0);
                let temp = self.builder.build_binary(IrBinaryOp::Eq, operand_val, zero, IrType::I1)?;
                Ok(Value::Temp(temp))
            }
            UnaryOp::BitNot => {
                // Generate XOR with -1 for bitwise NOT
                let operand_val = self.generate(operand)?;
                let all_ones = Value::Constant(-1);
                let result_type = IrType::I16;
                let temp = self.builder.build_binary(IrBinaryOp::Xor, operand_val, all_ones, result_type)?;
                Ok(Value::Temp(temp))
            }
            _ => {
                // TODO: Handle other unary ops
                Err(CodegenError::UnsupportedConstruct {
                    construct: format!("unary operator: {:?}", op),
                    location: operand.span.start.clone(),
                }.into())
            }
        }
    }
    
    /// Generate function call
    pub fn generate_function_call(&mut self, function: &Expression, arguments: &[Expression]) 
        -> Result<Value, CompilerError> {
        // Get function value
        let (func_val, _func_name) = match &function.kind {
            ExpressionKind::Identifier { name, .. } => {
                // Direct function call
                (Value::Global(name.clone()), name.clone())
            }
            _ => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: "indirect function calls".to_string(),
                    location: function.span.start.clone(),
                }.into());
            }
        };
        
        // Generate arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.generate(arg)?);
        }
        
        // Get return type from the function type
        let return_type = if let Some(expr_type) = &function.expr_type {
            match expr_type {
                Type::Function { return_type, .. } => {
                    convert_type(return_type, function.span.start.clone())?
                }
                _ => IrType::Void
            }
        } else {
            IrType::Void
        };
        
        if let Some(result_temp) = self.builder.build_call(func_val, arg_values, return_type)? {
            Ok(Value::Temp(result_temp))
        } else {
            Ok(Value::Constant(0)) // Void functions don't return a value
        }
    }
    
    /// Generate member access (object.member or pointer->member)
    pub fn generate_member_access(&mut self, object: &Expression, member: &str, is_pointer: bool) 
        -> Result<Value, CompilerError> {
        // Get the object type
        let obj_type = if let Some(ref expr_type) = object.expr_type {
            expr_type
        } else {
            return Err(CodegenError::InternalError {
                message: "Object has no type information".to_string(),
                location: object.span.start.clone(),
            }.into());
        };
        
        // Handle pointer dereferencing if needed
        let struct_type = if is_pointer {
            match obj_type {
                Type::Pointer(inner) => &**inner,
                _ => {
                    return Err(CodegenError::InternalError {
                        message: format!("Expected pointer type for -> operator, found {:?}", obj_type),
                        location: object.span.start.clone(),
                    }.into());
                }
            }
        } else {
            obj_type
        };
        
        // Extract struct fields
        let fields = match struct_type {
            Type::Struct { fields, .. } => fields,
            Type::Union { fields, .. } => fields,
            _ => {
                return Err(CodegenError::InternalError {
                    message: format!("Expected struct or union type, found {:?}", struct_type),
                    location: object.span.start.clone(),
                }.into());
            }
        };
        
        // Find the field and calculate offset
        let mut offset = 0u64;
        let mut field_type = None;
        for field in fields {
            if field.name == member {
                field_type = Some(&field.field_type);
                break;
            }
            // For struct, add field size to offset (no alignment for simplicity)
            // For union, offset is always 0
            if matches!(struct_type, Type::Struct { .. }) {
                if let Some(size) = field.field_type.size_in_words() {
                    offset += size;
                } else {
                    return Err(CodegenError::InternalError {
                        message: format!("Cannot compute size of field {}", field.name),
                        location: object.span.start.clone(),
                    }.into());
                }
            }
        }
        
        let field_type = field_type.ok_or_else(|| {
            CodegenError::UndefinedVariable {
                name: format!("field '{}'", member),
                location: object.span.start.clone(),
            }
        })?;
        
        // Generate the base address
        let base_addr = if is_pointer {
            // For pointer->member, load the pointer value
            self.generate(object)?
        } else {
            // For object.member, get the address of the object
            self.generate_lvalue(object)?
        };
        
        // Add offset if non-zero
        let field_addr = if offset > 0 {
            let offset_val = Value::Constant(offset as i64);
            let temp = self.builder.build_binary(IrBinaryOp::Add, base_addr, offset_val, 
                IrType::Ptr(Box::new(IrType::I16)))?;
            Value::Temp(temp)
        } else {
            base_addr
        };
        
        // Load the value
        let ir_field_type = convert_type(field_type, object.span.start.clone())?;
        let result = self.builder.build_load(field_addr, ir_field_type)?;
        Ok(Value::Temp(result))
    }
    
    /// Generate lvalue (address that can be assigned to)
    pub fn generate_lvalue(&mut self, expr: &Expression) -> Result<Value, CompilerError> {
        match &expr.kind {
            ExpressionKind::Identifier { name, .. } => {
                if let Some((value, _)) = self.variables.get(name) {
                    match value {
                        Value::Global(_) => Ok(value.clone()), // Global variables are already addresses
                        Value::Temp(_) => Ok(value.clone()),    // Local variables
                        _ => Err(CodegenError::InternalError {
                            message: "Invalid lvalue".to_string(),
                            location: expr.span.start.clone(),
                        }.into()),
                    }
                } else {
                    Err(CodegenError::UndefinedVariable {
                        name: name.clone(),
                        location: expr.span.start.clone(),
                    }.into())
                }
            }
            
            ExpressionKind::Unary { op: UnaryOp::Dereference, operand } => {
                // *ptr - the pointer itself is the address
                self.generate(operand)
            }
            
            ExpressionKind::Binary { op: BinaryOp::Index, left, right } => {
                // arr[idx] as lvalue - compute the address
                let base_ptr = self.generate(left)?;
                let index = self.generate(right)?;
                
                // Determine the element type from the array/pointer type
                let element_type = if let Some(expr_type) = &left.expr_type {
                    match expr_type {
                        Type::Array { element_type, .. } => {
                            convert_type(element_type, left.span.start.clone())?
                        }
                        Type::Pointer(target) => {
                            convert_type(target, left.span.start.clone())?
                        }
                        _ => IrType::I16 // Default to i16 for int
                    }
                } else {
                    IrType::I16 // Default to i16 for int
                };
                
                // Use GetElementPtr for proper array indexing
                let addr = self.builder.build_pointer_offset(
                    base_ptr,
                    index,
                    IrType::Ptr(Box::new(element_type))
                )?;
                
                Ok(addr)
            }
            
            ExpressionKind::Member { object, member, is_pointer } => {
                self.generate_member_lvalue(object, member, *is_pointer)
            }
            
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: "complex lvalue".to_string(),
                location: expr.span.start.clone(),
            }.into()),
        }
    }
    
    fn generate_member_lvalue(&mut self, object: &Expression, member: &str, is_pointer: bool) 
        -> Result<Value, CompilerError> {
        // Get struct type
        let obj_type = if let Some(ref expr_type) = object.expr_type {
            expr_type
        } else {
            return Err(CodegenError::InternalError {
                message: "Object has no type information".to_string(),
                location: object.span.start.clone(),
            }.into());
        };
        
        // Handle pointer dereferencing if needed
        let struct_type = if is_pointer {
            match obj_type {
                Type::Pointer(inner) => &**inner,
                _ => {
                    return Err(CodegenError::InternalError {
                        message: format!("Expected pointer type for -> operator, found {:?}", obj_type),
                        location: object.span.start.clone(),
                    }.into());
                }
            }
        } else {
            obj_type
        };
        
        // Extract struct fields
        let fields = match struct_type {
            Type::Struct { fields, .. } => fields,
            Type::Union { fields, .. } => fields,
            _ => {
                return Err(CodegenError::InternalError {
                    message: format!("Expected struct or union type, found {:?}", struct_type),
                    location: object.span.start.clone(),
                }.into());
            }
        };
        
        // Find the field and calculate offset
        let mut offset = 0u64;
        let mut found = false;
        for field in fields {
            if field.name == *member {
                found = true;
                break;
            }
            // For struct, add field size to offset
            if matches!(struct_type, Type::Struct { .. }) {
                if let Some(size) = field.field_type.size_in_words() {
                    offset += size;
                } else {
                    return Err(CodegenError::InternalError {
                        message: format!("Cannot compute size of field {}", field.name),
                        location: object.span.start.clone(),
                    }.into());
                }
            }
        }
        
        if !found {
            return Err(CodegenError::UndefinedVariable {
                name: format!("field '{}'", member),
                location: object.span.start.clone(),
            }.into());
        }
        
        // Generate the base address
        let base_addr = if is_pointer {
            // For pointer->member, load the pointer value
            self.generate(object)?
        } else {
            // For object.member, get the address of the object
            self.generate_lvalue(object)?
        };
        
        // Add offset if non-zero
        if offset > 0 {
            let offset_val = Value::Constant(offset as i64);
            let temp = self.builder.build_binary(IrBinaryOp::Add, base_addr, offset_val, 
                IrType::Ptr(Box::new(IrType::I16)))?;
            Ok(Value::Temp(temp))
        } else {
            Ok(base_addr)
        }
    }
}

/// Convert AST binary op to IR binary op
fn convert_binary_op(op: BinaryOp) -> Result<IrBinaryOp, String> {
    match op {
        BinaryOp::Add => Ok(IrBinaryOp::Add),
        BinaryOp::Sub => Ok(IrBinaryOp::Sub),
        BinaryOp::Mul => Ok(IrBinaryOp::Mul),
        BinaryOp::Div => Ok(IrBinaryOp::SDiv), // Use signed division for now
        BinaryOp::Mod => Ok(IrBinaryOp::SRem), // Use signed remainder for now
        BinaryOp::BitAnd => Ok(IrBinaryOp::And),
        BinaryOp::BitOr => Ok(IrBinaryOp::Or),
        BinaryOp::BitXor => Ok(IrBinaryOp::Xor),
        BinaryOp::LeftShift => Ok(IrBinaryOp::Shl),
        BinaryOp::RightShift => Ok(IrBinaryOp::AShr), // Arithmetic shift for signed
        BinaryOp::Less => Ok(IrBinaryOp::Slt),
        BinaryOp::Greater => Ok(IrBinaryOp::Sgt),
        BinaryOp::LessEqual => Ok(IrBinaryOp::Sle),
        BinaryOp::GreaterEqual => Ok(IrBinaryOp::Sge),
        BinaryOp::Equal => Ok(IrBinaryOp::Eq),
        BinaryOp::NotEqual => Ok(IrBinaryOp::Ne),
        // These are handled specially with short-circuit evaluation
        BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
            Err(format!("Logical operator {:?} should use short-circuit evaluation", op))
        }
        // Assignment operators are handled elsewhere
        BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign | 
        BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::ModAssign | 
        BinaryOp::BitAndAssign | BinaryOp::BitOrAssign | BinaryOp::BitXorAssign | 
        BinaryOp::LeftShiftAssign | BinaryOp::RightShiftAssign => {
            Err(format!("Assignment operator {:?} should be handled specially", op))
        }
        // Array indexing is handled specially
        BinaryOp::Index => {
            Err(format!("Index operator should be handled specially"))
        }
    }
}

