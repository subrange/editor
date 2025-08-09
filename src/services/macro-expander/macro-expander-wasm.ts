import type { MacroExpander, MacroExpanderOptions, MacroExpanderResult } from './macro-expander';
import init, { WasmMacroExpander } from '../../bf-macro-expander/pkg/bf_macro_expander';
import wasmUrl from '../../bf-macro-expander/pkg/bf_macro_expander_bg.wasm?url';

let wasmInitialized = false;
let wasmExpander: WasmMacroExpander | null = null;

async function ensureWasmInitialized(): Promise<WasmMacroExpander> {
  if (!wasmInitialized) {
    await init(wasmUrl);
    wasmInitialized = true;
  }
  
  if (!wasmExpander) {
    wasmExpander = new WasmMacroExpander();
  }
  
  return wasmExpander;
}

export class MacroExpanderWasm implements MacroExpander {
  private expanderPromise: Promise<WasmMacroExpander>;

  constructor() {
    this.expanderPromise = ensureWasmInitialized();
  }

  expand(input: string, options?: MacroExpanderOptions): MacroExpanderResult {
    // For synchronous interface, we'll need to handle this differently
    // The WASM version is async, so we'll need to either:
    // 1. Make this async
    // 2. Pre-initialize and throw if not ready
    // 3. Use a worker pattern
    
    // For now, let's throw if not initialized
    if (!wasmExpander) {
      throw new Error('WASM macro expander not initialized. Call initializeWasm() first.');
    }

    try {
      const wasmOptions = {
        strip_comments: options?.stripComments ?? false,
        collapse_empty_lines: options?.collapseEmptyLines ?? false,
        generate_source_map: options?.generateSourceMap ?? false,
        enable_circular_dependency_detection: options?.enableCircularDependencyDetection ?? true,
      };

      const result = wasmExpander.expand(input, wasmOptions);
      
      // Convert WASM source map to expected format if present
      let sourceMap = undefined;
      if (result.source_map) {
        // The WASM source map might have a different structure
        // For now, we'll skip source map support for WASM expander
        // TODO: Implement proper source map conversion
        // console.warn('Source map from WASM expander not yet supported');
      }
      
      // Map the result from WASM format to our format
      return {
        expanded: result.expanded || '',
        errors: result.errors || [],
        tokens: result.tokens || [],
        macros: result.macros || [],
        sourceMap: sourceMap,
      };
    } catch (error) {
      return {
        expanded: '',
        errors: [{
          type: 'syntax_error',
          message: error instanceof Error ? error.message : 'Unknown error during expansion',
        }],
        tokens: [],
        macros: [],
      };
    }
  }
}

export async function initializeWasm(): Promise<void> {
  await ensureWasmInitialized();
}

export function createMacroExpanderWasm(): MacroExpander {
  return new MacroExpanderWasm();
}