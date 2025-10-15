import { useMemo } from 'react';
import {
  ProgressiveMacroTokenizer,
  type MacroToken,
  progressiveMacroTokenStyles,
} from '../../editor/services/macro-tokenizer-progressive.ts';

interface BfMacroDisplayProps {
  content: string;
}

export function BfMacroDisplay({ content }: BfMacroDisplayProps) {
  const tokenizer = useMemo(() => new ProgressiveMacroTokenizer(), []);

  const tokenizedLines = useMemo(() => {
    const lines = content.split('\n');
    return lines.map((line, index) => tokenizer.tokenizeLine(line, index));
  }, [content, tokenizer]);

  return (
    <div className="font-mono text-xs">
      <div className="bg-zinc-900 rounded p-4 overflow-auto">
        {tokenizedLines.map((tokens, lineIndex) => (
          <div key={lineIndex} className="min-h-[1.25rem]">
            {tokens.length === 0 ? (
              <span>&nbsp;</span>
            ) : (
              tokens.map((token, tokenIndex) => (
                <span
                  key={tokenIndex}
                  className={
                    progressiveMacroTokenStyles[token.type] || 'text-zinc-300'
                  }
                >
                  {token.value}
                </span>
              ))
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
