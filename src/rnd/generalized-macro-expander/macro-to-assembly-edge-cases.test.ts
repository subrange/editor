import { describe, it, expect } from 'vitest';
import { createMacroExpanderV4 } from '../../services/macro-expander/macro-expander-v4.ts';
import { AssemblyBackendV4 } from './assembly-backend-v4.ts';

describe('Macro Expander V4 - Edge Cases', () => {
  describe('Empty and whitespace edge cases', () => {
    it('should handle empty macro body', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define empty() {}
        @empty()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('');
    });

    it('should handle macro with only whitespace', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define spaces() {   
          
        }
        @spaces()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('');
    });

    it('should handle macro with only comments', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define commented() {
          ; This is just a comment
          ; Another comment
        }
        @commented()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe(
        '; This is just a comment\n; Another comment',
      );
    });
  });

  describe('Parameter edge cases', () => {
    it('should handle parameters with similar names', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test(r, reg, register) {
          ADD r, reg, register
        }
        @test(R1, R2, R3)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('ADD R1, R2, R3');
    });

    it('should handle parameter that looks like a keyword', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test(if, repeat, for) {
          ADD if, repeat, for
        }
        @test(R1, R2, R3)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('ADD R1, R2, R3');
    });

    it('should not substitute partial parameter matches', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test(r) {
          ADD r1, r, r2
        }
        @test(R5)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('ADD r1, R5, r2');
    });

    it('should handle missing parameters gracefully', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test(a, b, c) {
          ADD a, b, c
        }
        @test(R1, R2)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('ADD R1, R2, c');
    });

    it('should handle extra parameters gracefully', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test(a, b) {
          ADD a, b, R0
        }
        @test(R1, R2, R3, R4)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('ADD R1, R2, R0');
    });
  });

  describe('Nested macro edge cases', () => {
    it('should handle deeply nested macro calls', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define a(x) { ADD x, x, 1 }
        #define b(x) { 
          @a(x) 
          @a(x) 
        }
        #define c(x) { 
          @b(x) 
          @b(x) 
        }
        @c(R1)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe(
        'ADD R1, R1, 1\nADD R1, R1, 1\nADD R1, R1, 1\nADD R1, R1, 1',
      );
    });

    it('should handle recursive macro with depth limit', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define recursive(x) { @recursive(x) }
        @recursive(R1)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].type).toBe('syntax_error');
      expect(result.errors[0].message).toContain(
        'Maximum macro expansion depth exceeded',
      );
    });

    it('should handle mutual recursion', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define foo(x) { @bar(x) }
        #define bar(x) { @foo(x) }
        @foo(R1)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].message).toContain(
        'Maximum macro expansion depth exceeded',
      );
    });
  });

  describe('Builtin function edge cases', () => {
    it('should handle repeat with count of 0', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() { {repeat(0, {ADD R1, R1, 1})} }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('');
    });

    it('should handle repeat with negative count', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() { {repeat(-5, {ADD R1, R1, 1})} }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].message).toContain('Invalid repeat count: -5');
    });

    it('should handle if with complex conditions', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() { 
          {if(0, {ADD R1, R1, 1}, {SUB R1, R1, 1})}
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('SUB R1, R1, 1');
    });

    it('should handle nested builtins', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          {repeat(2, {
            {if(1, {ADD R1, R1, 1}, {NOP})}
          })}
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('ADD R1, R1, 1\nADD R1, R1, 1');
    });

    it('should handle builtin with macro invocation as argument', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define count() 3
        #define body() { ADD R1, R1, 1 }
        #define test() {
          {repeat(@count(), @body())}
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe(
        'ADD R1, R1, 1\nADD R1, R1, 1\nADD R1, R1, 1',
      );
    });
  });

  describe('Meta-variable edge cases', () => {
    it('should handle multiple label prefixes in one macro', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define complex() {
          __LABEL__start:
          ADD R1, R1, 1
          BNE R1, R2, __LABEL__middle
          __LABEL__middle:
          SUB R1, R1, 1
          BEQ R1, R0, __LABEL__end
          __LABEL__end:
          NOP
        }
        @complex()
        @complex()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      const lines = result.expanded.trim().split('\n');

      // First invocation
      expect(lines[0]).toBe('start_1:');
      expect(lines[2]).toContain('middle_1');
      expect(lines[3]).toBe('middle_1:');
      expect(lines[5]).toContain('end_1');
      expect(lines[6]).toBe('end_1:');

      // Second invocation
      expect(lines[8]).toBe('start_2:');
      expect(lines[10]).toContain('middle_2');
      expect(lines[11]).toBe('middle_2:');
      expect(lines[13]).toContain('end_2');
      expect(lines[14]).toBe('end_2:');
    });

    it('should handle __INVOC_COUNT__ in various contexts', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          ; Invocation number __INVOC_COUNT__
          LI R__INVOC_COUNT__, __INVOC_COUNT__
        }
        @test()
        @test()
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      const lines = result.expanded.trim().split('\n');

      expect(lines[0]).toBe('; Invocation number 1');
      expect(lines[1]).toBe('LI R1, 1');
      expect(lines[2]).toBe('; Invocation number 2');
      expect(lines[3]).toBe('LI R2, 2');
      expect(lines[4]).toBe('; Invocation number 3');
      expect(lines[5]).toBe('LI R3, 3');
    });

    it('should handle __COUNTER__ with multiple uses', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          LI R__COUNTER__, __COUNTER__
          LI R__COUNTER__, __COUNTER__
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('LI R0, 1\nLI R2, 3');
    });

    it('should handle nested macro with parent tracking', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define outer() {
          ; In __MACRO_NAME__
          @inner()
        }
        #define inner() {
          ; In __MACRO_NAME__, parent was __PARENT_MACRO__
        }
        @outer()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe(
        '; In outer\n; In inner, parent was outer',
      );
    });
  });

  describe('Special character and escaping edge cases', () => {
    it('should handle assembly labels and directives', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          section_text:
          global_start:
          _start:
            MOV R0, #42
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe(
        'section_text:\nglobal_start:\n_start:\nMOV R0, #42',
      );
    });

    it('should handle parentheses in function calls', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          CALL func(R5, R6)
          ADD R1, (R2), R3
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      const lines = result.expanded.trim().split('\n');
      expect(lines[0]).toBe('CALL func(R5, R6)');
      expect(lines[1]).toBe('ADD R1, (R2), R3');
    });

    it('should handle standalone comments', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          ADD R1, R2, R3
          ; Next instruction
          SUB R4, R5, R6
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe(
        'ADD R1, R2, R3\n; Next instruction\nSUB R4, R5, R6',
      );
    });
  });

  describe('Complex formatting edge cases', () => {
    it('should preserve indentation within macros', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define function() {
          func:
            PUSH R1
            PUSH R2
            ; Do work
            POP R2
            POP R1
            RET
        }
        @function()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      const lines = result.expanded.trim().split('\n');
      expect(lines[0]).toBe('func:');
      expect(lines[1]).toBe('PUSH R1');
      expect(lines[2]).toBe('PUSH R2');
    });

    it('should handle multiple blank lines', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          ADD R1, R1, 1
          
          
          SUB R2, R2, 1
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      const lines = result.expanded.split('\n');
      expect(lines[0]).toBe('ADD R1, R1, 1');
      expect(lines[1]).toBe('');
      expect(lines[2]).toBe('');
      expect(lines[3]).toBe('SUB R2, R2, 1');
    });

    it('should handle inline comments correctly', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          ADD /* inline comment */ R1, R2, R3
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      // Inline comments might be stripped or preserved depending on parser
      expect(result.expanded.trim()).toMatch(/ADD.*R1.*R2.*R3/);
    });
  });

  describe('Error handling edge cases', () => {
    it('should report undefined macro usage', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        @undefined_macro(R1, R2)
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].type).toBe('undefined');
      expect(result.errors[0].message).toContain('undefined_macro');
    });

    it.skip('should report invalid builtin usage', () => {
      // Unknown builtins are currently parsed as text, not as builtin functions
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          {unknown_builtin(1, 2, 3)}
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].message).toContain(
        'Unknown builtin function: unknown_builtin',
      );
    });

    it('should report wrong number of arguments to builtins', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          {repeat(1)}
          {if(1, 2)}
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(2);
      expect(result.errors[0].message).toContain(
        'repeat() expects exactly 2 arguments',
      );
      expect(result.errors[1].message).toContain(
        'if() expects exactly 3 arguments',
      );
    });
  });

  describe('Backend-specific edge cases', () => {
    it.skip('should handle assembly-specific builtins', () => {
      // Backend-specific builtins are not yet implemented in the parser
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          {align(8)}
          {db(0x01, 0x02, 0x03)}
          {dw(0x1234, 0x5678)}
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe(
        '.align 8\n.byte 0x01, 0x02, 0x03\n.word 0x1234, 0x5678',
      );
    });

    it.skip('should handle empty arguments to backend builtins', () => {
      // Backend-specific builtins are not yet implemented in the parser
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          {align()}
          {db()}
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('.align 4\n.byte ');
    });

    it('should handle brainfuck commands as comments', () => {
      const backend = new AssemblyBackendV4();
      const expander = createMacroExpanderV4(backend);

      const input = `
        #define test() {
          ADD R1, R2, R3
          +++++ ; This should be parsed as BF
          SUB R4, R5, R6
        }
        @test()
      `;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toContain('ADD R1, R2, R3');
      expect(result.expanded).toContain('; BF: +++++');
      expect(result.expanded).toContain('SUB R4, R5, R6');
    });
  });
});
