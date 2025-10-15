import {
  useStoreSubscribe,
  useStoreSubscribeToField,
  useStoreSubscribeObservable,
} from '../../../hooks/use-store-subscribe.tsx';
import { EditorStore } from '../stores/editor.store.ts';
import { interpreterStore } from '../../debugger/interpreter-facade.store.ts';

interface LineNumbersPanelProps {
  store: EditorStore;
}

export function LineNumbersPanel({ store }: LineNumbersPanelProps) {
  const editorState = useStoreSubscribe(store.editorState);
  const expandedPosition = useStoreSubscribe(interpreterStore.currentChar);
  const sourcePosition = useStoreSubscribeObservable(
    interpreterStore.currentSourceChar,
    false,
    null,
  );
  const breakpoints = useStoreSubscribeToField(
    interpreterStore.state,
    'breakpoints',
  );

  // Use source position for macro editor when available, expanded position for main editor
  const isMacroEditor = store.getId() === 'macro';
  const currentChar =
    isMacroEditor && sourcePosition ? sourcePosition : expandedPosition;

  const handleLineClick = (lineIndex: number) => {
    if (!store.showDebug) {
      return;
    }

    const line = editorState.lines[lineIndex];
    if (!line) return;

    if (isMacroEditor) {
      // For macro editor, allow setting breakpoints on any non-empty line
      const trimmed = line.text.trim();
      if (trimmed.length > 0 && !trimmed.startsWith('//')) {
        // Set breakpoint.rs at first non-whitespace character
        const firstNonWhitespace = line.text.search(/\S/);
        if (firstNonWhitespace >= 0) {
          interpreterStore.toggleSourceBreakpoint({
            line: lineIndex,
            column: firstNonWhitespace,
          });
        }
      }
    } else {
      // For main editor, look for Brainfuck commands
      for (let i = 0; i < line.text.length; i++) {
        if ('><+-[].,$'.includes(line.text[i])) {
          interpreterStore.toggleBreakpoint({ line: lineIndex, column: i });
          break;
        }
      }
    }
  };

  return (
    <div className="flex flex-col overflow-visible bg-zinc-950 sticky left-0 w-16 min-w-16 min-h-0 text-zinc-700 select-none z-1 py-1">
      {editorState.lines.map((line, i) => {
        let hasBreakpoint = false;

        if (isMacroEditor) {
          // For macro editor, check if there's a breakpoint.rs on any non-empty, non-comment line
          const trimmed = line.text.trim();
          if (trimmed.length > 0 && !trimmed.startsWith('//')) {
            // Check at the first non-whitespace position
            const firstNonWhitespace = line.text.search(/\S/);
            if (firstNonWhitespace >= 0) {
              hasBreakpoint = interpreterStore.hasSourceBreakpointAt({
                line: i,
                column: firstNonWhitespace,
              });
            }
          }
        } else {
          // For main editor, check for BF commands
          hasBreakpoint = breakpoints.some((bp) => bp.line === i);
        }

        const isCurrentLine = currentChar.line === i;

        return (
          <div
            key={i}
            className={`flex justify-between align-center px-2  hover:bg-zinc-800 ${
              store.showDebug && isCurrentLine
                ? 'bg-zinc-800 text-zinc-300'
                : ''
            }`}
            onClick={() => handleLineClick(i)}
          >
            {store.showDebug && hasBreakpoint ? (
              <span className="text-red-500 mr-1">●</span>
            ) : (
              <span />
            )}
            {i + 1}
          </div>
        );
      })}
    </div>
  );
}
