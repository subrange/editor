Technical Task: Multi-Editor Architecture Refactoring
Overview
Refactor the Brainfuck IDE editor from a global singleton pattern to support multiple editor instances, enabling features like a dedicated macro editor. The implementation should use reactive stores (RxJS BehaviorSubjects) instead of React Context.
Objectives

Enable multiple concurrent editor instances
Maintain reactive state management without React Context
Support custom tokenizers per editor instance
Implement proper focus and keybinding isolation

Architecture Design
1. Editor Manager
   Create a centralized manager for editor instances:
   typescriptclass EditorManager {
   private editors: Map<string, EditorStore> = new Map();
   private activeEditor$: BehaviorSubject<string | null>;

createEditor(id: string, config?: EditorConfig): EditorStore;
getEditor(id: string): EditorStore | undefined;
setActiveEditor(id: string): void;
destroyEditor(id: string): void;
}
2. Editor Store Refactoring
   Transform the current singleton into an instantiable class:
   typescriptclass EditorStore {
   private id: string;
   private state$: BehaviorSubject<EditorState>;
   private tokenizer: ITokenizer;

constructor(id: string, config?: EditorConfig) {
this.id = id;
this.tokenizer = config?.tokenizer || new DefaultTokenizer();
// Initialize reactive state
}
}
3. Component Props Injection
   Replace direct store imports with prop drilling:
   typescriptinterface EditorProps {
   store: EditorStore;
   onFocus?: () => void;
   }

export const Editor: React.FC<EditorProps> = ({ store, onFocus }) => {
// Use store from props, not global import
};
Implementation Tasks
Phase 1: Core Refactoring

Create EditorManager Service

Implement editor lifecycle management
Handle active editor tracking
Expose observable for active editor changes


Refactor EditorStore

Remove singleton pattern
Add instance ID
Make all methods instance-based
Ensure proper cleanup on destroy


Update Editor Component

Accept store as prop
Remove global store imports
Pass store to all child components


Refactor Child Components

Update all editor child components to receive store via props
Remove direct imports of global editorStore
Use RxJS subscriptions with proper cleanup



Phase 2: Multi-Editor Support

Keybinding Isolation

Scope keybindings to active editor
Implement keybinding context switching
Handle focus management between editors


Focus Management

Track which editor has focus
Update EditorManager's activeEditor$ on focus changes
Ensure keybindings only apply to focused editor


Tokenizer Architecture

Define ITokenizer interface
Allow custom tokenizers per editor instance
Create MacroTokenizer extending base tokenizer



Phase 3: Integration

Update Main Application

Initialize EditorManager
Create main editor instance
Update existing code to use managed editor


Add Macro Editor

Create macro editor instance with custom tokenizer
Implement side-by-side layout
Handle macro-specific functionality



Technical Requirements
State Management

Use RxJS BehaviorSubjects for all state
No React Context usage
Proper subscription management with cleanup
Type-safe observables with TypeScript

Success Criteria

Multiple editor instances can run simultaneously
Each editor maintains independent state
Keybindings work correctly with focused editor only
No React Context usage in implementation
Custom tokenizers can be injected per editor
Existing functionality remains intact
Performance is not degraded
Memory leaks are prevented (proper cleanup)

Future Extensibility
The architecture should support:

Unlimited editor instances
Custom editor configurations
Different editor modes (read-only, macro, etc.)
Plugin system for editor extensions
Serialization/deserialization of editor state