import { useMemo, useRef, useState, useEffect } from 'react';
import {
  useStoreSubscribe,
  useStoreSubscribeToField,
  useStoreSubscribeObservable,
} from '../../../hooks/use-store-subscribe.tsx';
import { EditorStore, type Line } from '../stores/editor.store.ts';
import {
  ProgressiveMacroTokenizer,
  type MacroToken,
} from '../services/macro-tokenizer-progressive.ts';
import { AssemblyTokenizer } from '../services/assembly-tokenizer.ts';
import { ErrorDecorations } from './error-decorations.tsx';
import {
  type MacroExpansionError,
  type MacroDefinition,
} from '../../../services/macro-expander/macro-expander.ts';
import { MacroAutocomplete } from './macro-autocomplete.tsx';
import { AssemblyAutocompleteWrapper } from './assembly-autocomplete-wrapper.tsx';
import { AssemblyErrorDecorations } from './assembly-error-decorations.tsx';
import {
  LINE_PADDING_LEFT,
  LINE_PADDING_TOP,
  CHAR_HEIGHT,
} from '../constants.ts';
import { BracketHighlights } from './bracket-matcher.tsx';
import { VirtualizedLine } from './virtualized-line.tsx';
import { interpreterStore } from '../../debugger/interpreter-facade.store.ts';
import { SearchHighlights } from './search-highlights.tsx';
import { SearchScroll } from './search-scroll.tsx';
import { Selection } from './selection.tsx';
import { Cursor } from './cursor.tsx';
import { DebugMarker } from './debug-marker.tsx';
import { MacroUsagesModal, type MacroUsage } from './macro-usages-modal.tsx';
import { MacroRenameModal } from './macro-rename-modal.tsx';
import { UnusedMacroHighlights } from './unused-macro-highlights.tsx';
import { measureCharacterWidth } from '../../helpers.ts';

interface LinesPanelProps {
  store: EditorStore;
  editorWidth: number;
  scrollLeft: number;
  editorRef: React.RefObject<HTMLDivElement>;
}

export function LinesPanel({
  store,
  editorWidth,
  scrollLeft,
  editorRef,
}: LinesPanelProps) {
  const editorState = useStoreSubscribe(store.editorState);
  const lines = editorState.lines;
  const selection = editorState.selection;

  const containerRef = useRef<HTMLDivElement>(null);
  const charWidth = useMemo(() => measureCharacterWidth(), []);
  const isDraggingRef = useRef(false);
  const dragStartedRef = useRef(false);
  const [isMetaKeyHeld, setIsMetaKeyHeld] = useState(false);
  const [isShiftKeyHeld, setIsShiftKeyHeld] = useState(false);
  const [macroExpansionVersion, setMacroExpansionVersion] = useState(0);
  const [macroUsagesModal, setMacroUsagesModal] = useState<{
    macroName: string;
    usages: MacroUsage[];
  } | null>(null);
  const [macroRenameModal, setMacroRenameModal] = useState<{
    macroName: string;
    position: Position;
  } | null>(null);

  const breakpoints = useStoreSubscribeToField(
    interpreterStore.state,
    'breakpoints',
  );
  const expandedLine = useStoreSubscribeToField(
    interpreterStore.currentChar,
    'line',
  );
  const sourceLine = useStoreSubscribeObservable(
    interpreterStore.currentSourceChar,
    false,
    null,
  );
  const isRunning = useStoreSubscribeToField(
    interpreterStore.state,
    'isRunning',
  );

  // Use source position for macro editor when available
  const isMacroEditor = store.getId() === 'macro';
  const currentDebuggingLine =
    isMacroEditor && sourceLine ? sourceLine.line : expandedLine;

  // Get tokenizer from store
  const tokenizer = store.getTokenizer();

  // Subscribe to tokenizer state changes if it's an enhanced macro tokenizer
  useEffect(() => {
    if (tokenizer instanceof ProgressiveMacroTokenizer) {
      const unsubscribe = tokenizer.onStateChange(() => {
        // Force re-render by updating version
        setMacroExpansionVersion((v) => v + 1);
      });
      return unsubscribe;
    }
  }, [tokenizer]);

  // Tokenize all lines whenever content changes
  const tokenizedLines = useMemo(() => {
    const lineTexts = lines.map((l) => l.text);
    return tokenizer.tokenizeAllLines(lineTexts);
  }, [lines, tokenizer]); // Remove macroExpansionVersion - we don't need to re-tokenize

  // Determine which token styles to use based on tokenizer type
  const isProgressiveMacro = tokenizer instanceof ProgressiveMacroTokenizer;
  const isAssembly = tokenizer instanceof AssemblyTokenizer;

  // Extract errors and macros if using enhanced tokenizer
  const errors: MacroExpansionError[] = useMemo(() => {
    if (isProgressiveMacro && (tokenizer as ProgressiveMacroTokenizer).state) {
      const errs =
        (tokenizer as ProgressiveMacroTokenizer).state.expanderErrors || [];
      return errs;
    }
    return [];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isProgressiveMacro, tokenizer, macroExpansionVersion]); // macroExpansionVersion forces re-render when tokenizer state changes

  const availableMacros: MacroDefinition[] = useMemo(() => {
    if (isProgressiveMacro && (tokenizer as ProgressiveMacroTokenizer).state) {
      return (
        (tokenizer as ProgressiveMacroTokenizer).state.macroDefinitions || []
      );
    }
    return [];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isProgressiveMacro, tokenizer, macroExpansionVersion]); // macroExpansionVersion forces re-render when tokenizer state changes

  // Find which macros are unused
  const unusedMacros = useMemo(() => {
    const unused = new Set<string>();

    // First, collect all macro names
    availableMacros.forEach((macro) => {
      unused.add(macro.name);
    });

    // Then remove any that are used
    tokenizedLines.forEach((tokens) => {
      tokens.forEach((token) => {
        if (
          token.type === 'macro_invocation' ||
          token.type === 'hash_macro_invocation'
        ) {
          const macroName = token.value.match(/^[@#]([a-zA-Z_]\w*)/)?.[1];
          if (macroName) {
            unused.delete(macroName);
          }
        }
      });
    });

    return unused;
  }, [availableMacros, tokenizedLines]);

  // Function to find all usages of a macro
  const findMacroUsages = (macroName: string): MacroUsage[] => {
    const usages: MacroUsage[] = [];

    // Search through all tokenized lines
    tokenizedLines.forEach((tokens, lineIndex) => {
      tokens.forEach((token) => {
        if (
          token.type === 'macro_invocation' ||
          token.type === 'hash_macro_invocation'
        ) {
          // Extract the macro name from the token value (remove @ or # and any parameters)
          const match = token.value.match(/^([@#])([a-zA-Z_]\w*)/);
          if (match && match[2] === macroName) {
            usages.push({
              line: lineIndex,
              column: token.start,
              text: lines[lineIndex].text.trim(),
              lineNumber: `${lineIndex + 1}`,
              prefix: match[1] as '@' | '#',
            });
          }
        }
      });
    });

    return usages;
  };

  // Function to rename a macro throughout the file
  const renameMacro = (oldName: string, newName: string) => {
    // Find all occurrences to replace
    const replacements: Array<{
      start: Position;
      end: Position;
      text: string;
    }> = [];

    // Find macro definition by looking for macro_name tokens
    tokenizedLines.forEach((tokens, lineIndex) => {
      tokens.forEach((token) => {
        if (token.type === 'macro_name' && token.value === oldName) {
          // This is the macro name in the definition
          replacements.push({
            start: { line: lineIndex, column: token.start },
            end: { line: lineIndex, column: token.end },
            text: newName,
          });
        }
      });
    });

    // Find all invocations (@ or #)
    tokenizedLines.forEach((tokens, lineIndex) => {
      tokens.forEach((token) => {
        if (
          token.type === 'macro_invocation' ||
          token.type === 'hash_macro_invocation'
        ) {
          const tokenMacroName = token.value.match(/^[@#]([a-zA-Z_]\w*)/)?.[1];
          if (tokenMacroName === oldName) {
            // Replace just the name part after @ or #
            replacements.push({
              start: { line: lineIndex, column: token.start + 1 }, // Skip the @ or #
              end: {
                line: lineIndex,
                column: token.start + 1 + oldName.length,
              },
              text: newName,
            });
          }
        }
      });
    });

    // Use the new batchReplace method for atomic undo/redo
    store.batchReplace(replacements);
  };

  // Track cmd/ctrl key state and handle F2 for rename
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        setIsMetaKeyHeld(true);
      }
      if (e.shiftKey) {
        setIsShiftKeyHeld(true);
      }

      // Handle F2 for rename
      if (e.key === 'F2' && isProgressiveMacro) {
        e.preventDefault();
        e.stopPropagation();

        // Get current cursor position
        const cursorPos = store.editorState.value.selection.focus;
        const line = lines[cursorPos.line];
        if (!line) return;

        // Get token at cursor position
        const tokens = tokenizedLines[cursorPos.line] || [];
        const tokenAtCursor = tokens.find(
          (token) =>
            token.start <= cursorPos.column && token.end > cursorPos.column,
        );

        if (tokenAtCursor) {
          if (
            tokenAtCursor.type === 'macro_invocation' ||
            tokenAtCursor.type === 'hash_macro_invocation'
          ) {
            // Extract macro name from invocation (@ or #)
            const macroName =
              tokenAtCursor.value.match(/^[@#]([a-zA-Z_]\w*)/)?.[1];
            if (macroName) {
              setMacroRenameModal({
                macroName,
                position: cursorPos,
              });
            }
          } else if (tokenAtCursor.type === 'macro_name') {
            // Direct macro name in definition
            setMacroRenameModal({
              macroName: tokenAtCursor.value,
              position: cursorPos,
            });
          }
        }
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (!e.metaKey && !e.ctrlKey) {
        setIsMetaKeyHeld(false);
      }
      if (!e.shiftKey) {
        setIsShiftKeyHeld(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [isProgressiveMacro, lines, tokenizedLines, store]);

  // Helper to convert mouse position to text position
  const getPositionFromMouse = (e: React.MouseEvent) => {
    if (!containerRef.current) return null;

    const rect = containerRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left - LINE_PADDING_LEFT;
    const y = e.clientY - rect.top - LINE_PADDING_TOP;

    // Calculate line number
    let line = Math.floor(y / CHAR_HEIGHT);
    line = Math.max(0, Math.min(line, lines.length - 1));

    // Calculate column
    let column = Math.round(x / charWidth);
    column = Math.max(0, Math.min(column, lines[line].text.length));

    return { line, column };
  };

  const handleClick = (e: React.MouseEvent) => {
    // Ignore click if we just finished dragging
    if (isDraggingRef.current) {
      isDraggingRef.current = false;
      return;
    }

    // Only handle single clicks (not part of double/triple click)
    if (e.detail === 1) {
      const position = getPositionFromMouse(e);
      if (!position) return;

      // Check if shift is held for extending selection
      if (e.shiftKey) {
        store.updateSelection(position);
      } else {
        store.setCursorPosition(position);
      }
    }
  };

  const handleDoubleClick = (e: React.MouseEvent) => {
    const position = getPositionFromMouse(e);
    if (!position) return;

    store.selectWord(position);
  };

  const handleTripleClick = (e: React.MouseEvent) => {
    const position = getPositionFromMouse(e);
    if (!position) return;

    store.selectLine(position.line);
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    // Only start drag selection on single click
    if (e.detail !== 1) return;
    // And on the left mouse button
    if (e.button !== 0) return;

    const position = getPositionFromMouse(e);
    if (!position) return;

    // Don't start new selection if shift is held
    if (!e.shiftKey) {
      store.startSelection(position);
    }

    dragStartedRef.current = false;

    // Add mouse move and up listeners for selection
    const handleMouseMove = (e: MouseEvent) => {
      const rect = containerRef.current?.getBoundingClientRect();
      if (!rect) return;

      // Mark that we're actually dragging (not just clicking)
      if (!dragStartedRef.current) {
        dragStartedRef.current = true;
        isDraggingRef.current = true;
      }

      const x = e.clientX - rect.left - LINE_PADDING_LEFT;
      const y = e.clientY - rect.top - LINE_PADDING_TOP;

      let line = Math.floor(y / CHAR_HEIGHT);
      line = Math.max(0, Math.min(line, lines.length - 1));

      let column = Math.round(x / charWidth);
      column = Math.max(0, Math.min(column, lines[line].text.length));

      store.updateSelection({ line, column });
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);

      // Only set isDraggingRef if we actually moved the mouse
      if (!dragStartedRef.current) {
        isDraggingRef.current = false;
      }
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  const handleTokenClick = (e: React.MouseEvent, token: MacroToken) => {
    // Handle assembly label navigation
    if (isAssembly && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      e.stopPropagation();

      const assemblyToken = token as any; // Token might be AssemblyToken

      // Handle label reference click - jump to label definition
      if (assemblyToken.type === 'label_ref') {
        const labelName = assemblyToken.value;

        // Find the label definition
        for (let i = 0; i < lines.length; i++) {
          const labelMatch = lines[i].text.match(/^([a-zA-Z_][a-zA-Z0-9_]*):/);
          if (labelMatch && labelMatch[1] === labelName) {
            // Set navigation flag for center scrolling
            store.isNavigating.next(true);
            // Jump to the label definition
            store.setCursorPosition({ line: i, column: 0 });
            break;
          }
        }
      }
      // Handle label definition click - show usages
      else if (assemblyToken.type === 'label') {
        const labelName = assemblyToken.value.slice(0, -1); // Remove colon

        // Find all usages of this label
        const usages: MacroUsage[] = [];
        for (let i = 0; i < lines.length; i++) {
          const line = lines[i].text;
          let index = 0;

          // Skip the definition line
          if (line.match(new RegExp(`^${labelName}:`))) {
            continue;
          }

          // Find all occurrences of the label in the line
          while ((index = line.indexOf(labelName, index)) !== -1) {
            // Check if it's a whole word (not part of another identifier)
            const before = index > 0 ? line[index - 1] : ' ';
            const after =
              index + labelName.length < line.length
                ? line[index + labelName.length]
                : ' ';

            if (/\W/.test(before) && /\W/.test(after)) {
              usages.push({
                line: i,
                column: index,
                text: line.trim(),
                lineNumber: (i + 1).toString(),
              });
            }
            index += labelName.length;
          }
        }

        // Show the modal with usages
        setMacroUsagesModal({
          macroName: labelName,
          usages,
        });
      }
      return;
    }

    if (!isProgressiveMacro) {
      return;
    }

    // Handle Shift+Click for rename
    if (
      e.shiftKey &&
      (token.type === 'macro_invocation' ||
        token.type === 'hash_macro_invocation' ||
        token.type === 'macro_name')
    ) {
      e.preventDefault();
      e.stopPropagation();

      let macroName: string | undefined;
      if (
        token.type === 'macro_invocation' ||
        token.type === 'hash_macro_invocation'
      ) {
        macroName = token.value.match(/^[@#]([a-zA-Z_]\w*)/)?.[1];
      } else {
        macroName = token.value;
      }

      if (macroName) {
        setMacroRenameModal({
          macroName,
          position: { line: 0, column: 0 }, // Position not used currently
        });
      }
      return;
    }

    // Handle Cmd/Ctrl+Click for navigation
    if (!(e.metaKey || e.ctrlKey)) {
      return;
    }

    e.preventDefault();
    e.stopPropagation();

    // Check if we're clicking on a macro invocation (@ or #)
    if (
      token.type === 'macro_invocation' ||
      token.type === 'hash_macro_invocation'
    ) {
      // Extract macro name from the token value (remove @ or # and parameters)
      const macroName = token.value.match(/^[@#]([a-zA-Z_]\w*)/)?.[1];
      if (!macroName) {
        return;
      }

      // Find the macro definition
      const macroDef = availableMacros.find((m) => m.name === macroName);
      if (macroDef && macroDef.sourceLocation) {
        // Set navigation flag for center scrolling
        store.isNavigating.next(true);
        // Jump to the macro definition
        store.setCursorPosition({
          line: macroDef.sourceLocation.line,
          column: macroDef.sourceLocation.column,
        });
      }
    }
    // Check if we're clicking on a macro name in a definition
    else if (token.type === 'macro_name') {
      const macroName = token.value;
      const usages = findMacroUsages(macroName);

      // Show the modal with usages
      setMacroUsagesModal({
        macroName,
        usages,
      });
    }
  };

  const renderLine = (line: Line, lineIndex: number) => {
    const tokens = tokenizedLines[lineIndex] || [];
    let hasBreakpoint = false;

    if (isMacroEditor) {
      // For macro editor, use the same logic as line numbers panel
      const trimmed = line.text.trim();
      if (trimmed.length > 0 && !trimmed.startsWith('//')) {
        const firstNonWhitespace = line.text.search(/\S/);
        if (firstNonWhitespace >= 0) {
          hasBreakpoint = interpreterStore.hasSourceBreakpointAt({
            line: lineIndex,
            column: firstNonWhitespace,
          });
        }
      }
    } else {
      // For main editor, check regular breakpoints
      hasBreakpoint = breakpoints.some((bp) => bp.line === lineIndex);
    }

    const isCurrentLine = currentDebuggingLine === lineIndex;

    return (
      <VirtualizedLine
        key={lineIndex}
        tokens={tokens}
        lineText={line.text}
        lineIndex={lineIndex}
        charWidth={charWidth}
        isProgressiveMacro={isProgressiveMacro}
        isAssembly={isAssembly}
        hasBreakpoint={hasBreakpoint}
        isCurrentLine={isCurrentLine}
        isRunning={isRunning}
        showDebug={store.showDebug}
        onTokenClick={handleTokenClick}
        isMetaKeyHeld={isMetaKeyHeld}
        isShiftKeyHeld={isShiftKeyHeld}
        editorWidth={editorWidth || 1000}
        editorScrollLeft={scrollLeft}
      />
    );
  };

  return (
    <div
      ref={containerRef}
      className="flex flex-col grow-1 overflow-visible py-1 relative cursor-text min-h-0 pb-24"
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      onMouseDown={handleMouseDown}
      onMouseUp={(e) => {
        if (e.detail === 3) {
          handleTripleClick(e);
        }
      }}
    >
      <div className="">{lines.map(renderLine)}</div>
      <Selection store={store} />
      <BracketHighlights
        cursorPosition={selection.focus}
        lines={lines}
        charWidth={charWidth}
      />
      <SearchHighlights searchStore={store.searchStore} charWidth={charWidth} />
      <SearchScroll searchStore={store.searchStore} charWidth={charWidth} />
      {isProgressiveMacro && errors.length > 0 && (
        <ErrorDecorations store={store} errors={errors} />
      )}
      {isProgressiveMacro && (
        <>
          <MacroAutocomplete
            store={store}
            macros={availableMacros}
            charWidth={charWidth}
          />
          <UnusedMacroHighlights
            unusedMacros={unusedMacros}
            tokenizedLines={tokenizedLines}
            charWidth={charWidth}
          />
        </>
      )}
      {isAssembly && (
        <>
          <AssemblyAutocompleteWrapper store={store} charWidth={charWidth} />
          <AssemblyErrorDecorations store={store} charWidth={charWidth} />
        </>
      )}
      <Cursor store={store} />
      {store.showDebug && <DebugMarker store={store} />}
      {macroUsagesModal && (
        <MacroUsagesModal
          macroName={macroUsagesModal.macroName}
          usages={macroUsagesModal.usages}
          isOpen={true}
          onClose={() => {
            setMacroUsagesModal(null);
            // Focus the editor when modal is closed
            setTimeout(() => {
              editorRef.current?.focus();
            }, 0);
          }}
          onNavigate={(usage) => {
            // Close modal first to ensure proper focus handling
            setMacroUsagesModal(null);

            // Then navigate after modal is closed
            setTimeout(() => {
              // Focus the editor - this should trigger onFocus handler
              if (editorRef.current) {
                editorRef.current.focus();
                // Force focus event if needed
                editorRef.current.dispatchEvent(
                  new FocusEvent('focus', { bubbles: true }),
                );
              }

              // Then navigate after another small delay
              setTimeout(() => {
                // Set navigation flag for center scrolling
                store.isNavigating.next(true);
                // Jump to the usage location
                store.setCursorPosition({
                  line: usage.line,
                  column: usage.column,
                });
              }, 0);
            }, 0);
          }}
        />
      )}
      {macroRenameModal && (
        <MacroRenameModal
          isOpen={true}
          currentName={macroRenameModal.macroName}
          onClose={() => {
            setMacroRenameModal(null);
            // Focus the editor when modal is closed
            setTimeout(() => {
              editorRef.current?.focus();
            }, 0);
          }}
          onRename={(newName) => {
            renameMacro(macroRenameModal.macroName, newName);
            setMacroRenameModal(null);
            // Focus the editor after rename
            setTimeout(() => {
              editorRef.current?.focus();
            }, 0);
          }}
          existingMacroNames={availableMacros.map((m) => m.name)}
        />
      )}
    </div>
  );
}
