# Product Requirements Document: Macro Source Maps for Brainfuck IDE

## Executive Summary

This PRD outlines the implementation of source map support for the Brainfuck IDE's macro system. Currently, when debugging macro-enabled Brainfuck programs, the debugger highlights positions in the expanded Brainfuck code rather than the original macro source. This feature will enable users to see their original macro code highlighted during execution, providing a more intuitive debugging experience.

## Problem Statement

### Current State
- Users write Brainfuck programs using macros for better code organization and reusability
- During debugging, the IDE shows execution position in the **expanded** Brainfuck code
- Users must mentally map between their macro source and the expanded output
- Breakpoints can only be set on expanded code lines

### User Pain Points
1. **Cognitive Load**: Difficult to understand which macro is currently executing
2. **Debugging Complexity**: Hard to trace execution flow through macro calls
3. **Lost Context**: No visibility into macro parameters or expansion context during execution
4. **Confusing Breakpoints**: Breakpoints set on expanded code don't persist across re-expansions

## Goals and Success Criteria

### Primary Goals
1. Show original macro source line highlighting during program execution
2. Enable setting breakpoints on macro source lines
3. Provide macro expansion context during debugging
4. Maintain performance and responsiveness of the IDE

### Success Criteria
- Users can debug macro code without viewing expanded output
- Execution highlighting accurately tracks through macro calls and expansions
- Source map generation adds <100ms to typical macro expansion times
- Memory overhead for source maps is <10% of expanded code size

## User Stories

### As a Brainfuck developer using macros
1. I want to see my cursor in the original macro code during debugging, so I can understand program flow in terms of my abstractions
2. I want to set breakpoints on macro definition lines, so they persist across code changes
3. I want to see which macro is currently executing and its parameter values, so I can debug macro logic

## Functional Requirements

### 1. Source Map Generation

The macro expander must generate position mappings between source and expanded code:

```typescript
interface SourceMapEntry {
  // Position in expanded code
  expandedRange: {
    start: { line: number; column: number };
    end: { line: number; column: number };
  };
  
  // Position in source code
  sourceRange: {
    start: { line: number; column: number };
    end: { line: number; column: number };
  };
  
  // Macro context
  macroName?: string;
  macroCallSite?: Position;
  expansionDepth: number;
  parameterValues?: Record<string, string>;
}
```

### 2. Position Translation

The interpreter must track dual positions during execution:
- Current position in expanded code (for actual execution)
- Corresponding position in source code (for UI display)
- Macro expansion context at current position

### 3. UI Integration

#### Editor Display
- Highlight current line in source code during debugging
- Show macro expansion indicator in gutter
- Display macro context in status bar or tooltip

#### Breakpoint System
- Allow setting breakpoints on macro source lines
- Translate source breakpoints to expanded positions
- Persist breakpoints across macro re-expansions
- Visual distinction between source and expanded breakpoints

#### Debug Information Panel
- Show current macro name and parameters
- Display call stack for nested macro expansions
- Option to view expanded code at current position

### 4. Configuration Options

Users should be able to configure:
- Source map detail level (full vs simplified)
- Macro context display preferences
- Performance vs accuracy trade-offs

## Technical Requirements

### 1. Macro Expander Modifications

#### Position Tracking
- Track character-level positions during all expansion operations
- Handle multi-line macro definitions and expansions
- Account for whitespace normalization and line endings

#### Mapping Generation
- Create bidirectional lookup structures
- Optimize for common access patterns (forward during expansion, reverse during debugging)
- Support incremental updates for partial re-expansions

### 2. Interpreter Integration

#### State Management
- Store source map alongside expanded code
- Efficient position lookup during execution
- Cache frequently accessed mappings

#### Performance Requirements
- Position translation must take <1ms per lookup
- Source map size should be <50% of expanded code size
- No noticeable impact on execution speed

### 3. Data Persistence

- Source maps should be included in file snapshots
- Support for source map versioning
- Graceful handling of outdated mappings

## Implementation Phases

### Phase 1: Core Source Map Generation (Week 1-2)
- Modify macro expander to track positions
- Generate basic source-to-expanded mappings
- Handle simple macro expansions

### Phase 2: Complex Mapping Support (Week 2-3)
- Handle nested macro expansions
- Support built-in functions (repeat, for, if)
- Optimize mapping data structures

### Phase 3: Interpreter Integration (Week 3-4)
- Add source position tracking to interpreter
- Implement position translation utilities
- Update debugging state management

### Phase 4: Basic UI Support (Week 4-5)
- Source line highlighting during debugging
- Simple macro context display
- Toggle between source/expanded views

### Phase 5: Advanced Features (Week 5-6)
- Source-level breakpoints
- Macro call stack visualization
- Hover tooltips with expansion info
- Performance optimizations

## Edge Cases and Considerations

### Technical Challenges
1. **Many-to-One Mappings**: Multiple source lines expanding to single output
2. **One-to-Many Mappings**: Single macro call expanding to many lines
3. **Recursive Macros**: Handling infinite expansion prevention
4. **Dynamic Expansions**: Macros that generate different output based on parameters

### Error Handling
- Graceful degradation when source maps are unavailable
- Clear error messages for mapping conflicts
- Fallback to expanded view on mapping errors

### Performance Considerations
- Lazy source map generation for large files
- Incremental updates for small changes
- Memory-efficient storage formats
- Background processing in web workers

## Success Metrics

### Performance Metrics
- Source map generation time: <100ms for 1000-line files
- Position lookup time: <1ms average, <5ms worst case
- Memory overhead: <10MB for typical programs
- UI responsiveness: No frame drops during debugging

### User Experience Metrics
- Reduced debugging time for macro-heavy programs
- Increased usage of macro features
- Positive user feedback on debugging experience
- Reduced support requests about macro debugging

## Future Enhancements

### Potential Extensions
1. **Visual Macro Expansion**: Show inline previews of macro expansions
2. **Macro Profiling**: Performance metrics for macro expansions
3. **Source Map Export**: Standard format for external tool integration
4. **Macro Testing Framework**: Unit tests for individual macros

## Conclusion

Implementing source maps for the Brainfuck IDE's macro system will significantly improve the debugging experience for users working with macro-enabled programs. By showing execution in the context of the original macro source rather than expanded code, we reduce cognitive load and make the development process more intuitive. The phased implementation approach ensures we can deliver value incrementally while maintaining system stability and performance.