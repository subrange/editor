// First, add a bracket matcher utility
import { useMemo } from 'react';
import {
  CHAR_HEIGHT,
  LINE_PADDING_LEFT,
  LINE_PADDING_TOP,
} from '../constants.ts';

interface BracketPair {
  open: string;
  close: string;
}

class BracketMatcher {
  private pairs: BracketPair[] = [
    { open: '[', close: ']' },
    { open: '{', close: '}' },
    { open: '(', close: ')' },
  ];

  private openBrackets = new Set(this.pairs.map((p) => p.open));
  private closeBrackets = new Set(this.pairs.map((p) => p.close));
  private bracketMap = new Map([
    ...this.pairs.map((p) => [p.open, p.close] as const),
    ...this.pairs.map((p) => [p.close, p.open] as const),
  ]);

  findMatchingBracket(
    lines: string[],
    position: { line: number; column: number },
  ): { line: number; column: number } | null {
    const line = lines[position.line];
    if (!line || position.column >= line.length) return null;

    const char = line[position.column];
    const isOpen = this.openBrackets.has(char);
    const isClose = this.closeBrackets.has(char);

    if (!isOpen && !isClose) return null;

    const matchingChar = this.bracketMap.get(char);
    if (!matchingChar) return null;

    if (isOpen) {
      return this.findClosingBracket(lines, position, char, matchingChar);
    } else {
      return this.findOpeningBracket(lines, position, char, matchingChar);
    }
  }

  private findClosingBracket(
    lines: string[],
    start: { line: number; column: number },
    openChar: string,
    closeChar: string,
  ): { line: number; column: number } | null {
    let depth = 1;
    let line = start.line;
    let column = start.column + 1;

    while (line < lines.length) {
      const text = lines[line];

      while (column < text.length) {
        const char = text[column];

        if (char === openChar) {
          depth++;
        } else if (char === closeChar) {
          depth--;
          if (depth === 0) {
            return { line, column };
          }
        }

        column++;
      }

      line++;
      column = 0;
    }

    return null;
  }

  private findOpeningBracket(
    lines: string[],
    start: { line: number; column: number },
    closeChar: string,
    openChar: string,
  ): { line: number; column: number } | null {
    let depth = 1;
    let line = start.line;
    let column = start.column - 1;

    while (line >= 0) {
      const text = lines[line];

      if (column < 0) {
        line--;
        if (line >= 0) {
          column = lines[line].length - 1;
        }
        continue;
      }

      while (column >= 0) {
        const char = text[column];

        if (char === closeChar) {
          depth++;
        } else if (char === openChar) {
          depth--;
          if (depth === 0) {
            return { line, column };
          }
        }

        column--;
      }

      line--;
      if (line >= 0) {
        column = lines[line].length - 1;
      }
    }

    return null;
  }
}

// Component to render bracket highlights
export function BracketHighlights({
  cursorPosition,
  lines,
  charWidth,
}: {
  cursorPosition: { line: number; column: number };
  lines: { text: string }[];
  charWidth: number;
}) {
  const matcher = useMemo(() => new BracketMatcher(), []);

  const brackets = useMemo(() => {
    const lineTexts = lines.map((l) => l.text);
    const positions: Array<{ line: number; column: number }> = [];

    // Check cursor position
    const match = matcher.findMatchingBracket(lineTexts, cursorPosition);
    if (match) {
      positions.push(cursorPosition);
      positions.push(match);
    }

    // Also check position before cursor (common when typing)
    if (cursorPosition.column > 0) {
      const beforeCursor = {
        line: cursorPosition.line,
        column: cursorPosition.column - 1,
      };
      const matchBefore = matcher.findMatchingBracket(lineTexts, beforeCursor);
      if (matchBefore) {
        positions.push(beforeCursor);
        positions.push(matchBefore);
      }
    }

    return positions;
  }, [lines, cursorPosition, matcher]);

  return (
    <>
      {brackets.map((pos, index) => (
        <div
          key={`${pos.line}-${pos.column}-${index}`}
          className="absolute bg-yellow-400 opacity-40 pointer-events-none"
          style={{
            left: `${LINE_PADDING_LEFT + pos.column * charWidth}px`,
            top: `${LINE_PADDING_TOP + pos.line * CHAR_HEIGHT - 3}px`,
            width: `${charWidth}px`,
            height: `${CHAR_HEIGHT}px`,
          }}
        />
      ))}
    </>
  );
}
