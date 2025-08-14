# Type System Prerequisites Roadmap

## Executive Summary

Before implementing the pointer arithmetic features outlined in POINTER_ARITHMETIC_ROADMAP.md, we need to establish a complete type system foundation. This document outlines the prerequisite features that must be implemented first.

**UPDATE (Dec 2024)**: Analysis reveals that much of the symbol table infrastructure already exists but is disconnected from the typed AST conversion layer!

**Key Discovery**: 
- ✅ Symbol type tracking exists: `SemanticAnalyzer::symbol_types`
- ✅ Typedef support exists: `SemanticAnalyzer::type_definitions`  
- ✅ Type resolution works in semantic analysis
- ❌ But `TypeEnvironment` can't access any of this!

**Bottom Line**: Phase 1 is now just a 1-2 day wiring task instead of a week-long implementation!

## Current State Analysis (UPDATED)

### ✅ What Already Exists (More Than Expected!)

1. **Symbol Type Tracking**
   - `SemanticAnalyzer` has `symbol_types: HashMap<SymbolId, Type>`
   - Stores types for all functions, globals, parameters, and local variables
   - `ExpressionAnalyzer::analyze()` fills in `expr_type` on all expressions
   - Symbol lookup and type assignment works during semantic analysis

2. **Typedef Support** 
   - `type_definitions: HashMap<String, Type>` stores typedef mappings
   - `SymbolManager::declare_global_variable()` handles `typedef` storage class
   - `TypeAnalyzer::resolve_type()` resolves typedef names to concrete types
   - Already functional in semantic analysis!

3. **Symbol Resolution**
   - `SymbolTable` from rcc_common provides scoped symbol management
   - Symbols are properly tracked with enter_scope()/exit_scope()
   - Symbol IDs are assigned and stored in AST nodes

### 🔴 Critical Blockers

1. ~~**TypeEnvironment Not Connected to symbol_types**~~ ✅ FIXED
   - ~~`TypeEnvironment` in typed_ast/conversion.rs is just an empty struct `{}`~~
   - ~~Cannot access the `symbol_types` HashMap from semantic analysis~~
   - ~~This breaks the entire typed AST conversion process!~~

2. **No Cast Expression Support**
   - Parser cannot parse `(type)expression` - treats all `(...)` as parenthesized expressions
   - Codegen explicitly errors on Cast expressions
   - Blocks all void pointer usage and type conversions

3. **Incomplete Struct Implementation**
   - Parser can parse struct declarations
   - No struct layout calculation for field offsets
   - Member access parsing incomplete
   - Blocks struct field access via GEP

### 🟡 Partial Implementations

1. **Type System**
   - ✅ Basic types defined (int, char, pointers, etc.)
   - ✅ Type::is_assignable_from() for compatibility
   - ✅ Symbol types tracked in semantic analysis
   - ❌ TypeEnvironment can't access symbol_types

2. **Type Checking**
   - ✅ TypeChecker classifies binary operations
   - ✅ Recognizes pointer arithmetic patterns
   - ✅ ExpressionAnalyzer assigns types to all expressions
   - ❌ No cast expression handling

3. **Member Access**
   - ✅ AST has Member variant
   - ✅ TypedExpr::MemberAccess defined
   - ❌ Parser doesn't fully handle member expressions
   - ❌ No struct layout for offset calculation

## Implementation Roadmap (REVISED)

### Phase 1: Connect TypeEnvironment to Existing Symbol Tables ✅ COMPLETED

#### Implementation Summary
Phase 1 has been successfully completed! The TypeEnvironment now properly connects to the existing symbol tables.

#### What Was Done
1. **Added getter methods to SemanticAnalyzer** (`into_type_info()`) to expose symbol_types and type_definitions
2. **Updated TypeEnvironment** from empty struct to hold actual type mappings
3. **Modified compilation pipeline** to pass type information from semantic analysis to typed AST conversion
4. **Updated all existing tests** to use the new API
5. **Created comprehensive test suite** with 13 tests covering various type scenarios

#### Key Discoveries During Implementation

##### The Typedef Parser Challenge
During implementation, we discovered a fundamental limitation: **typedef'd names cannot be used in variable declarations** because the parser doesn't have access to the typedef table. 

Example that fails:
```c
typedef int myint;
myint x = 42;  // Parser error: expects ';' after 'myint'
```

The parser sees `myint` as an identifier (potential variable/function name), not a type specifier. This is a classic C parsing problem - you need semantic information during parsing to distinguish typedef names from other identifiers.

**Impact**: Typedefs are properly stored and can be resolved, but cannot yet be used in declarations. This would require either:
- Making typedef table available to the parser (complex)
- Using a lexer hack to mark typedef names as special tokens
- Two-pass parsing

##### What Works Now
- ✅ Variable type lookups through symbol_types  
- ✅ Type information flows correctly from semantic analysis to typed AST
- ✅ Typedef definitions are stored and accessible
- ✅ All existing tests pass
- ✅ 11 out of 13 new tests pass (2 document typedef limitation)

##### Files Modified
- `rcc-frontend/src/typed_ast/conversion.rs` - TypeEnvironment implementation
- `rcc-frontend/src/semantic/mod.rs` - Added getter methods
- `rcc-frontend/src/lib.rs` - Updated compilation pipeline
- `rcc-frontend/src/codegen/mod.rs` - Updated test calls
- `rcc-frontend/src/codegen_tests.rs` - Updated test calls
- `rcc-frontend/src/type_environment_tests.rs` - New comprehensive test suite

**Actual Time**: ~2 hours (even faster than estimated!)

### Phase 2: Cast Expression Support

#### Why This Is Critical
- Required for void pointer usage (`(int*)void_ptr`)
- Needed for NULL implementation (`(void*)0`)
- Essential for type conversions
- Blocks test_pointers_evil.c at line 23

#### Implementation Tasks

##### Task 2.1: Parser Support for Cast Expressions
**File**: `rcc-frontend/src/parser/expressions.rs`

Modify `parse_primary_expression()` to detect cast vs parenthesized expression:

```rust
Some(Token { token_type: TokenType::LeftParen, .. }) => {
    // Look ahead to determine if this is a cast or parenthesized expr
    if self.is_type_start() {
        // Parse cast expression
        let target_type = self.parse_type_name()?;
        self.expect(TokenType::RightParen, "cast expression")?;
        let operand = self.parse_unary_expression()?;
        ExpressionKind::Cast {
            target_type,
            operand: Box::new(operand),
        }
    } else {
        // Parse parenthesized expression
        let expr = self.parse_expression()?;
        self.expect(TokenType::RightParen, "parenthesized expression")?;
        return Ok(expr);
    }
}
```

##### Task 2.2: Type Name Parsing
**File**: `rcc-frontend/src/parser/types.rs`

Add method to parse type names in cast expressions:
```rust
pub fn parse_type_name(&mut self) -> Result<Type, CompilerError> {
    let base_type = self.parse_type_specifier()?;
    // Handle abstract declarators (*, [], etc. without identifier)
    self.parse_abstract_declarator(base_type)
}

pub fn is_type_start(&self) -> bool {
    matches!(self.peek().map(|t| &t.token_type), Some(
        TokenType::Void | TokenType::Char | TokenType::Int |
        TokenType::Short | TokenType::Long | TokenType::Unsigned |
        TokenType::Signed | TokenType::Struct | TokenType::Union |
        TokenType::Enum | TokenType::Identifier(_) // Could be typedef
    ))
}
```

##### Task 2.3: Codegen for Cast Expressions
**File**: `rcc-frontend/src/codegen/expressions/mod.rs`

Replace the error with actual implementation:
```rust
TypedExpr::Cast { operand, target_type, .. } => {
    let operand_val = self.generate(operand)?;
    let source_type = operand.get_type();
    
    match (source_type, target_type) {
        // Pointer to pointer cast (including void*)
        (Type::Pointer { .. }, Type::Pointer { .. }) => {
            // Fat pointers: may need to adjust bank tag
            Ok(operand_val) // For now, just pass through
        }
        // Integer to pointer cast
        (t, Type::Pointer { .. }) if t.is_integer() => {
            // Convert integer to fat pointer
            self.builder.build_int_to_ptr(operand_val, target_type)
        }
        // Pointer to integer cast
        (Type::Pointer { .. }, t) if t.is_integer() => {
            // Extract address component from fat pointer
            self.builder.build_ptr_to_int(operand_val, target_type)
        }
        // Integer to integer cast
        (s, t) if s.is_integer() && t.is_integer() => {
            // Handle sign extension/truncation
            self.builder.build_int_cast(operand_val, source_type, target_type)
        }
        _ => Err(CodegenError::InvalidCast { source_type, target_type })
    }
}
```

### Phase 3: Struct Support

#### Why Structs Are Needed
- Required for struct field access via GEP (Phase 2.3 of pointer arithmetic)
- Common in real C code
- Need layout calculation for correct offsets

#### Implementation Tasks

##### Task 3.1: Struct Layout Calculation
**File**: `rcc-frontend/src/semantic/types.rs`

```rust
pub fn calculate_struct_layout(fields: &[StructField]) -> StructLayout {
    let mut offset = 0;
    let mut field_infos = Vec::new();
    
    for field in fields {
        let size = field.field_type.size_in_words().unwrap_or(1);
        field_infos.push(FieldInfo {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
            offset,
        });
        offset += size;
    }
    
    StructLayout {
        fields: field_infos,
        total_size: offset,
    }
}
```

##### Task 3.2: Member Access Parsing
**File**: `rcc-frontend/src/parser/expressions.rs`

Already partially implemented but needs completion in postfix expression parsing.

##### Task 3.3: Member Access Codegen
**File**: `rcc-frontend/src/codegen/expressions/mod.rs`

```rust
TypedExpr::MemberAccess { object, member, offset, is_pointer, .. } => {
    let object_val = self.generate(object)?;
    
    let ptr_val = if *is_pointer {
        // Already a pointer (->)
        object_val
    } else {
        // Need address of object (.)
        self.builder.build_address_of(object_val)?
    };
    
    // Use GEP with constant offset
    let offset_val = Value::Constant(*offset as i64);
    let result_ptr = self.builder.build_pointer_offset(
        ptr_val, 
        offset_val, 
        member_type
    )?;
    
    // Load the value
    self.builder.build_load(result_ptr, member_type)
}
```

### Phase 4: Typedef Support

#### Why Typedef Is Important
- Required for type aliases
- Common in C code (size_t, FILE, etc.)
- Needed for clean type resolution

#### Implementation Tasks

##### Task 4.1: Typedef Registration
During semantic analysis, register typedefs in symbol table.

##### Task 4.2: Typedef Resolution
When encountering an identifier in type position, check if it's a typedef.

##### Task 4.3: Update Parser
Parser needs to distinguish typedef names from regular identifiers.

### Phase 5: NULL Support

#### Implementation Approach
1. Define NULL as a macro or builtin: `#define NULL ((void*)0)`
2. Or recognize literal 0 in pointer context as NULL
3. Ensure void* cast works (requires Phase 2)

## Testing Strategy

### Test Order
1. **Symbol table tests** - Variable type lookups
2. **Cast expression tests** - Type conversions
3. **Struct tests** - Member access and layout
4. **Typedef tests** - Type aliases
5. **Integration tests** - Combined features

### Key Test Files to Enable
- `test_pointers_evil.c` - Requires casts, void pointers
- `test_struct_simple.c` - Requires struct support
- Future pointer arithmetic tests from POINTER_ARITHMETIC_ROADMAP.md

## Implementation Order (REVISED)

### Recommended Sequence
1. **Phase 1**: Connect TypeEnvironment (1-2 days!)
   - Just wire existing symbol_types to TypeEnvironment
   - Immediately unblocks type lookups
   - Typedef support already works!

2. **Phase 2**: Cast Expressions (3-4 days)
   - Parser changes to recognize cast syntax
   - Codegen implementation for type conversions
   - Makes test_pointers_evil.c progress further

3. **Phase 3**: Struct Support (1 week)
   - Struct layout calculation
   - Complete member access parsing
   - Required for GEP field access

4. **Phase 4**: NULL Support (1 day)
   - Simple once casts work
   - Define as `(void*)0` or recognize 0 in pointer context

**Total Estimate**: ~2 weeks (reduced from 3 weeks!)

### Major Discovery
- **Typedef support already exists** in semantic analysis - just needs to be passed through!
- **Symbol type tracking is complete** - just disconnected from typed AST layer
- Phase 1 is now a simple wiring task instead of building from scratch

## Success Criteria

### Must Have
- ✅ Symbol table tracks all variable types
- ✅ Cast expressions parse and generate correct code
- ✅ Struct member access works with correct offsets
- ✅ Typedef names resolve correctly
- ✅ test_pointers_evil.c compiles and runs

### Should Have
- ✅ NULL recognized as `(void*)0`
- ✅ Proper error messages for type mismatches
- ✅ Support for anonymous structs
- ✅ Nested struct support

### Nice to Have
- Union support
- Enum support beyond basic integers
- Better typedef scoping rules
- Type qualifiers (const, volatile)

## Relationship to Pointer Arithmetic

Once these prerequisites are complete, the pointer arithmetic implementation can proceed because:

1. **Type lookups work** - Can determine if expression is pointer
2. **Casts work** - Can handle void* and type conversions  
3. **Structs work** - Can generate GEP for field access
4. **Full type info available** - Can properly scale pointer arithmetic

This forms the foundation that POINTER_ARITHMETIC_ROADMAP.md assumes exists.

## Conclusion

The pointer arithmetic roadmap is blocked not by pointer arithmetic itself, but by fundamental type system gaps. This roadmap addresses those gaps in a logical order, starting with the symbol table foundation and building up to full struct and typedef support.

**Key Takeaway**: Implement this roadmap first, then the pointer arithmetic roadmap becomes straightforward to complete.