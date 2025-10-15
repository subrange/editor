import { type ITokenizer } from '../../../services/editor-manager.service.ts';
import {
  getAvailableMnemonics,
  getAvailableRegisters,
} from '../../../services/ripple-assembler/assembler.ts';
import { BehaviorSubject } from 'rxjs';

// Token types for assembly syntax
export interface AssemblyToken {
  type:
    | 'directive'
    | 'instruction'
    | 'register'
    | 'number'
    | 'label'
    | 'label_ref'
    | 'string'
    | 'comment'
    | 'mark_comment'
    | 'operator'
    | 'punctuation'
    | 'whitespace'
    | 'unknown'
    | 'error';
  value: string;
  start: number;
  end: number;
}

// Store for tokenizer state
interface TokenizerState {
  instructions: Set<string>;
  registers: Set<string>;
  initialized: boolean;
}

// Initial state with defaults
const initialState: TokenizerState = {
  instructions: new Set([
    'NOP',
    'ADD',
    'SUB',
    'AND',
    'OR',
    'XOR',
    'SLL',
    'SRL',
    'SLT',
    'SLTU',
    'ADDI',
    'ANDI',
    'ORI',
    'XORI',
    'LI',
    'SLLI',
    'SRLI',
    'LOAD',
    'STORE',
    'JAL',
    'JALR',
    'BEQ',
    'BNE',
    'BLT',
    'BGE',
    'BRK',
    'MUL',
    'DIV',
    'MOD',
    'MULI',
    'DIVI',
    'MODI',
    // Pseudo-instructions
    'HALT',
    'MOVE',
    'PUSH',
    'POP',
    'CALL',
    'RET',
    'INC',
    'DEC',
    'NEG',
    'NOT',
  ]),
  registers: new Set([
    'R0',
    'PC',
    'PCB',
    'RA',
    'RAB',
    'RV0',
    'RV1',
    'A0',
    'A1',
    'A2',
    'A3',
    'X0',
    'X1',
    'X2',
    'X3',
    'T0',
    'T1',
    'T2',
    'T3',
    'T4',
    'T5',
    'T6',
    'T7',
    'S0',
    'S1',
    'S2',
    'S3',
    'SC',
    'SB',
    'SP',
    'FP',
    'GP',
    // Also include R-prefixed versions for compatibility
    'R3',
    'R4',
    'R5',
    'R6',
    'R7',
    'R8',
    'R9',
    'R10',
    'R11',
    'R12',
    'R13',
    'R14',
    'R15',
    'R16',
    'R17',
    'R18',
    'R19',
    'R20',
    'R21',
    'R22',
    'R23',
    'R24',
    'R25',
    'R26',
    'R27',
    'R28',
    'R29',
    'R30',
    'R31',
  ]),
  initialized: false,
};

// RxJS store for tokenizer state
const tokenizerState$ = new BehaviorSubject<TokenizerState>(initialState);

// Current state references for quick access
let INSTRUCTIONS = initialState.instructions;
let REGISTERS = initialState.registers;

// Initialize instruction and register sets from WASM
async function initializeSets() {
  const state = tokenizerState$.value;
  if (state.initialized) return;

  try {
    const [mnemonics, registers] = await Promise.all([
      getAvailableMnemonics(),
      getAvailableRegisters(),
    ]);

    // Update with dynamic values from WASM
    INSTRUCTIONS = new Set(mnemonics.map((m) => m.toUpperCase()));
    REGISTERS = new Set(registers.map((r) => r.toUpperCase()));

    // Update the store
    tokenizerState$.next({
      instructions: INSTRUCTIONS,
      registers: REGISTERS,
      initialized: true,
    });

    console.log('Assembly tokenizer: Loaded dynamic instruction set from WASM');
  } catch (error) {
    console.warn(
      'Assembly tokenizer: Using default instruction set (WASM not loaded yet)',
      error,
    );
    // Keep using the defaults that were already set
  }
}

// Export the observable for components to subscribe to
export const assemblyTokenizerState$ = tokenizerState$.asObservable();

// Try to initialize on module load (but don't wait for it)
initializeSets();

// Directives
const DIRECTIVES = new Set([
  '.data',
  '.code',
  '.space',
  '.byte',
  '.word',
  '.asciiz',
  '.ascii',
]);

export class AssemblyTokenizer implements ITokenizer {
  private labels: Set<string> = new Set();
  private inDataSection = false;
  private inCodeSection = false;

  constructor() {
    // Ensure sets are initialized when tokenizer is created
    initializeSets();
  }

  reset() {
    this.labels.clear();
    this.inDataSection = false;
    this.inCodeSection = false;
  }

  tokenizeLine(
    text: string,
    _lineIndex: number,
    _isLastLine: boolean = false,
  ): AssemblyToken[] {
    const tokens: AssemblyToken[] = [];
    let position = 0;

    // Check for section directives
    if (text.trim() === '.data') {
      this.inDataSection = true;
      this.inCodeSection = false;
    } else if (text.trim() === '.code') {
      this.inDataSection = false;
      this.inCodeSection = true;
    }

    while (position < text.length) {
      let matched = false;

      // Skip whitespace
      if (!matched) {
        const wsMatch = text.slice(position).match(/^\s+/);
        if (wsMatch) {
          tokens.push({
            type: 'whitespace',
            value: wsMatch[0],
            start: position,
            end: position + wsMatch[0].length,
          });
          position += wsMatch[0].length;
          matched = true;
        }
      }

      // Comments (including // MARK: support)
      if (!matched && text[position] === ';') {
        const commentText = text.slice(position);
        tokens.push({
          type: 'comment',
          value: commentText,
          start: position,
          end: text.length,
        });
        return tokens;
      }

      if (!matched && text.slice(position, position + 2) === '//') {
        const commentText = text.slice(position);
        const isMarkComment = /^\/\/\s*MARK:/i.test(commentText);
        tokens.push({
          type: isMarkComment ? 'mark_comment' : 'comment',
          value: commentText,
          start: position,
          end: text.length,
        });
        return tokens;
      }

      // Directives
      if (!matched) {
        const directiveMatch = text.slice(position).match(/^\.[a-zA-Z]+/);
        if (directiveMatch && DIRECTIVES.has(directiveMatch[0].toUpperCase())) {
          tokens.push({
            type: 'directive',
            value: directiveMatch[0],
            start: position,
            end: position + directiveMatch[0].length,
          });
          position += directiveMatch[0].length;
          matched = true;
        }
      }

      // Labels (identifier followed by colon)
      if (!matched) {
        const labelMatch = text
          .slice(position)
          .match(/^[a-zA-Z_][a-zA-Z0-9_]*:/);
        if (labelMatch) {
          const labelName = labelMatch[0].slice(0, -1); // Remove colon
          this.labels.add(labelName);
          tokens.push({
            type: 'label',
            value: labelMatch[0],
            start: position,
            end: position + labelMatch[0].length,
          });
          position += labelMatch[0].length;
          matched = true;
        }
      }

      // Instructions
      if (!matched) {
        const instrMatch = text.slice(position).match(/^[a-zA-Z]+/);
        if (instrMatch && INSTRUCTIONS.has(instrMatch[0].toUpperCase())) {
          tokens.push({
            type: 'instruction',
            value: instrMatch[0],
            start: position,
            end: position + instrMatch[0].length,
          });
          position += instrMatch[0].length;
          matched = true;
        }
      }

      // Registers
      if (!matched) {
        // Match all register patterns: R0-R31, PC, PCB, RA, RAB, RV0-1, A0-3, X0-3, T0-7, S0-3, SC, SB, SP, FP, GP
        const regMatch = text
          .slice(position)
          .match(
            /^(R\d{1,2}|PC|PCB|RA|RAB|RV[01]|A[0-3]|X[0-3]|T[0-7]|S[0-3C]|SB|SP|FP|GP|V[01])\b/i,
          );
        if (regMatch && REGISTERS.has(regMatch[0].toUpperCase())) {
          tokens.push({
            type: 'register',
            value: regMatch[0],
            start: position,
            end: position + regMatch[0].length,
          });
          position += regMatch[0].length;
          matched = true;
        }
      }

      // Numbers (decimal, hex, binary, character literals)
      if (!matched) {
        const hexMatch = text.slice(position).match(/^0[xX][0-9a-fA-F]+/);
        const binMatch = text.slice(position).match(/^0[bB][01]+/);
        const charMatch = text.slice(position).match(/^'(\\.|[^'])'/);
        const decMatch = text.slice(position).match(/^-?\d+/);

        if (hexMatch) {
          tokens.push({
            type: 'number',
            value: hexMatch[0],
            start: position,
            end: position + hexMatch[0].length,
          });
          position += hexMatch[0].length;
          matched = true;
        } else if (binMatch) {
          tokens.push({
            type: 'number',
            value: binMatch[0],
            start: position,
            end: position + binMatch[0].length,
          });
          position += binMatch[0].length;
          matched = true;
        } else if (charMatch) {
          tokens.push({
            type: 'number',
            value: charMatch[0],
            start: position,
            end: position + charMatch[0].length,
          });
          position += charMatch[0].length;
          matched = true;
        } else if (decMatch) {
          tokens.push({
            type: 'number',
            value: decMatch[0],
            start: position,
            end: position + decMatch[0].length,
          });
          position += decMatch[0].length;
          matched = true;
        }
      }

      // Strings
      if (!matched && text[position] === '"') {
        const stringMatch = text.slice(position).match(/^"([^"\\]|\\.)*"/);
        if (stringMatch) {
          tokens.push({
            type: 'string',
            value: stringMatch[0],
            start: position,
            end: position + stringMatch[0].length,
          });
          position += stringMatch[0].length;
          matched = true;
        }
      }

      // Label references (identifiers that match known labels)
      if (!matched) {
        const identMatch = text
          .slice(position)
          .match(/^[a-zA-Z_][a-zA-Z0-9_]*/);
        if (identMatch) {
          // Check if it's a known label
          const isLabel = this.labels.has(identMatch[0]);
          tokens.push({
            type: isLabel ? 'label_ref' : 'unknown',
            value: identMatch[0],
            start: position,
            end: position + identMatch[0].length,
          });
          position += identMatch[0].length;
          matched = true;
        }
      }

      // Operators and punctuation
      if (!matched) {
        const opMatch = text.slice(position).match(/^[,+\-*/%]/);
        if (opMatch) {
          tokens.push({
            type: text[position] === ',' ? 'punctuation' : 'operator',
            value: opMatch[0],
            start: position,
            end: position + 1,
          });
          position++;
          matched = true;
        }
      }

      // Unknown character
      if (!matched) {
        tokens.push({
          type: 'unknown',
          value: text[position],
          start: position,
          end: position + 1,
        });
        position++;
      }
    }

    return tokens;
  }

  tokenizeAllLines(lines: string[]): AssemblyToken[][] {
    // Reset state for full document tokenization
    this.reset();

    // First pass - collect all labels
    lines.forEach((line) => {
      const labelMatch = line.match(/^[a-zA-Z_][a-zA-Z0-9_]*:/);
      if (labelMatch) {
        this.labels.add(labelMatch[0].slice(0, -1));
      }
    });

    // Second pass - tokenize with label knowledge
    return lines.map((line, index) =>
      this.tokenizeLine(line, index, index === lines.length - 1),
    );
  }
}

// Token styles for assembly syntax
export const assemblyTokenStyles: Record<AssemblyToken['type'], string> = {
  directive: 'text-purple-400/85',
  instruction: 'text-blue-300/85',
  register: 'text-green-400/75',
  number: 'text-amber-400/80',
  label: 'text-rose-400/85',
  label_ref: 'text-rose-300/75 italic',
  string: 'text-orange-300/80',
  comment: 'text-gray-400/85 italic',
  mark_comment: 'text-yellow-300 bg-yellow-900/30',
  operator: 'text-cyan-400/85',
  punctuation: 'text-zinc-400/70',
  whitespace: '',
  unknown: 'text-gray-500/75',
  error: 'text-red-300/90 underline decoration-wavy',
};
