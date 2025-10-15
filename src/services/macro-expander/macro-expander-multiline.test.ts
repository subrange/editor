import { describe, it, expect } from 'vitest';
import { createMacroExpander } from './macro-expander.ts';

describe('MacroExpander - Multiline Macro Support', () => {
  describe('Basic multiline macros with braces', () => {
    it('should support multiline macro definition with braces', () => {
      const input = `#define somemacro {
        // this is a comment
        ++++
        // another comment
        ----
      }
      @somemacro`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++----');
    });

    it('should allow empty lines in multiline macros', () => {
      const input = `#define test {
        >
        
        <
        
        +
      }
      @test`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('><+');
    });

    it('should support nested macro calls in multiline macros', () => {
      const input = `#define inner ++
      #define outer {
        @inner
        --
        @inner
      }
      @outer`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++--++');
    });

    it('should support builtin functions in multiline macros', () => {
      const input = `#define complex {
        // Initialize
        [-]
        
        // Set value
        {repeat(5, +)}
        
        // Move
        >
      }
      @complex`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('[-]+++++>');
    });

    it('should support parameterized multiline macros', () => {
      const input = `#define set(n) {
        // Clear current cell
        [-]
        
        // Set to n
        {repeat(n, +)}
      }
      @set(3)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('[-]+++');
    });

    it('should handle multiline macros with complex expressions', () => {
      const input = `#define loop(arr) {
        // Process each element
        {for(x in arr, 
          // Process x
          {repeat(x, >)}
          [-]
          {repeat(x, <)}
        )}
      }
      @loop({1, 2, 3})`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('>[-]<>>[-]<<>>>[-]<<<');
    });

    it('should support both old and new syntax in the same file', () => {
      const input = `#define old_style +\\
-\\
>
#define new_style {
  +
  -
  <
}
@old_style @new_style`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+-> +-<');
    });

    it('should report error for unclosed multiline macro', () => {
      const input = `#define bad {
        ++++
        // missing closing brace`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('closing brace');
    });

    it('should handle nested braces in multiline macros', () => {
      const input = `#define nested {
        {repeat(3, +>)}
        <
      }
      @nested`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+>+>+><');
    });

    it('should preserve indentation style in multiline macros', () => {
      const input = `#define formatted {
        [-]    // Clear
        +++    // Add 3
        >      // Next
      }
      @formatted`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('[-]+++>');
    });

    it('should handle inline comments after opening brace', () => {
      const input = `#define test { // Start of macro
        ++++
      }
      @test`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++');
    });

    it('should handle single-line brace macros', () => {
      const input = `#define simple { ++++ }
      @simple`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++');
    });

    it('should handle empty brace macros', () => {
      const input = `#define empty { }
      #define empty2 {
      }
      @empty @empty2`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('');
    });

    it('should support conditional logic in multiline macros', () => {
      const input = `#define check(val) {
        // Check if val is non-zero
        {if(val, +++>, ---<)}
      }
      @check(1) @check(0)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+++> ---<');
    });

    it('should handle multiline macros with array operations', () => {
      const input = `#define process_array(arr) {
        // Process in reverse
        {for(n in {reverse(arr)}, 
          {repeat(n, +)}
          .
        )}
      }
      @process_array({1, 2, 3})`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+++.++.+.');
    });
  });

  describe('Edge cases and error handling', () => {
    it('should handle brace in comment', () => {
      const input = `#define test {
        // This is not a closing }
        ++++
      }
      @test`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++');
    });

    it('should handle brace in string context', () => {
      const input = `#define test {
        // Using braces in BF: }{ are just text
        +{-}+
      }
      @test`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      // The {-} in the middle is treated as text since it's not a valid builtin
      expect(result.expanded.trim()).toContain('+');
    });

    it('should support mixing parameter styles', () => {
      const input = `#define fn(a, b) {
        // Use both params
        {repeat(a, +)}
        >
        {repeat(b, -)}
      }
      @fn(2, 3)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++>---');
    });
  });
});
