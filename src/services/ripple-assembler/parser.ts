import { ParsedLine } from './types';

export class Parser {
  private caseInsensitive: boolean;

  constructor(caseInsensitive: boolean = true) {
    this.caseInsensitive = caseInsensitive;
  }

  parseSource(source: string): ParsedLine[] {
    const lines = source.split('\n');
    const parsed: ParsedLine[] = [];

    for (let i = 0; i < lines.length; i++) {
      const line = this.parseLine(lines[i], i + 1);
      if (line.label || line.mnemonic || line.directive) {
        parsed.push(line);
      }
    }

    return parsed;
  }

  private parseLine(line: string, lineNumber: number): ParsedLine {
    const raw = line;
    
    const commentIndex = line.indexOf(';');
    if (commentIndex !== -1) {
      line = line.substring(0, commentIndex);
    }
    
    const hashCommentIndex = line.indexOf('#');
    if (hashCommentIndex !== -1) {
      line = line.substring(0, hashCommentIndex);
    }
    
    const doubleSlashIndex = line.indexOf('//');
    if (doubleSlashIndex !== -1) {
      line = line.substring(0, doubleSlashIndex);
    }

    line = line.trim();

    const result: ParsedLine = {
      operands: [],
      lineNumber,
      raw
    };

    if (!line) {
      return result;
    }

    // Check for directives first (start with .)
    if (line.startsWith('.')) {
      const tokens = this.tokenize(line);
      if (tokens.length > 0) {
        result.directive = tokens[0].substring(1).toLowerCase(); // Remove . and lowercase
        result.directiveArgs = tokens.slice(1);
      }
      return result;
    }

    const colonIndex = line.indexOf(':');
    if (colonIndex !== -1) {
      result.label = line.substring(0, colonIndex).trim();
      line = line.substring(colonIndex + 1).trim();
    }

    if (!line) {
      return result;
    }

    // Check for directives after label
    if (line.startsWith('.')) {
      const tokens = this.tokenize(line);
      if (tokens.length > 0) {
        result.directive = tokens[0].substring(1).toLowerCase();
        result.directiveArgs = tokens.slice(1);
      }
      return result;
    }

    const tokens = this.tokenize(line);
    if (tokens.length > 0) {
      result.mnemonic = this.caseInsensitive ? tokens[0].toUpperCase() : tokens[0];
      result.operands = tokens.slice(1);
    }

    return result;
  }

  private tokenize(line: string): string[] {
    const tokens: string[] = [];
    let current = '';
    let inString = false;
    let stringChar = '';

    for (let i = 0; i < line.length; i++) {
      const char = line[i];

      if (inString) {
        if (char === stringChar && line[i - 1] !== '\\') {
          inString = false;
          current += char;
        } else {
          current += char;
        }
      } else {
        if (char === '"' || char === "'") {
          inString = true;
          stringChar = char;
          current += char;
        } else if (/[\s,]/.test(char)) {
          if (current) {
            tokens.push(current);
            current = '';
          }
        } else {
          current += char;
        }
      }
    }

    if (current) {
      tokens.push(current);
    }

    return tokens;
  }

  parseDirective(line: string): { directive: string; args: string[] } | null {
    const trimmed = line.trim();
    
    if (!trimmed.startsWith('.') && !trimmed.startsWith('@')) {
      return null;
    }

    const tokens = this.tokenize(trimmed);
    if (tokens.length === 0) {
      return null;
    }

    return {
      directive: tokens[0],
      args: tokens.slice(1)
    };
  }

  parseMacroCall(text: string): { name: string; args: string[] } | null {
    const match = text.match(/^@(\w+)\s*\((.*)\)$/);
    if (!match) {
      return null;
    }

    const name = match[1];
    const argsString = match[2];
    
    const args = this.parseMacroArgs(argsString);
    
    return { name, args };
  }

  private parseMacroArgs(argsString: string): string[] {
    const args: string[] = [];
    let current = '';
    let depth = 0;
    let inString = false;
    let stringChar = '';

    for (let i = 0; i < argsString.length; i++) {
      const char = argsString[i];

      if (inString) {
        if (char === stringChar && argsString[i - 1] !== '\\') {
          inString = false;
        }
        current += char;
      } else {
        if (char === '"' || char === "'") {
          inString = true;
          stringChar = char;
          current += char;
        } else if (char === '(') {
          depth++;
          current += char;
        } else if (char === ')') {
          depth--;
          current += char;
        } else if (char === ',' && depth === 0) {
          args.push(current.trim());
          current = '';
        } else {
          current += char;
        }
      }
    }

    if (current.trim()) {
      args.push(current.trim());
    }

    return args;
  }
}