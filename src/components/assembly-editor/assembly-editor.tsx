import { useEffect, useState, useCallback, useMemo } from 'react';
import { Editor } from '../editor/editor.tsx';
import { editorManager } from '../../services/editor-manager.service.ts';
import {
  AssemblyTokenizer,
  assemblyTokenizerState$,
} from '../editor/services/assembly-tokenizer.ts';
import {
  CpuChipIcon,
  DocumentTextIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/solid';
import { IconButton } from '../ui/icon-button.tsx';
import { settingsStore } from '../../stores/settings.store.ts';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe.tsx';
import { assemblyOutputStore } from '../../stores/assembly-output.store.ts';
import { AssemblyOutput } from './assembly-output.tsx';
import { DraggableVSep } from '../ui/draggable-vsep.tsx';
import { useLocalStorageState } from '../../hooks/use-local-storage-state.tsx';
import { EditorStore } from '../editor/stores/editor.store.ts';
import {
  createAssembler,
  initAssembler,
} from '../../services/ripple-assembler/assembler.ts';
import {
  AssemblyQuickNavStore,
  type AssemblyNavigationItem,
} from './stores/assembly-quick-nav.store.ts';
import { AssemblyQuickNav } from './components/assembly-quick-nav.tsx';
import { HSep } from '../helper-components.tsx';
import { assemblyToMacroService } from '../../services/assembly-to-macro.service.ts';

export function AssemblyEditor() {
  const [assemblyEditor, setAssemblyEditor] = useState<EditorStore | null>(
    () => editorManager.getEditor('assembly') || null,
  );
  const [showOutput, setShowOutput] = useLocalStorageState(
    'assemblyShowOutput',
    false,
  );
  const [leftPanelWidth, setLeftPanelWidth] = useLocalStorageState(
    'assemblyLeftPanelWidth',
    60,
  );
  const settings = useStoreSubscribe(settingsStore.settings);
  const autoCompile = settings?.assembly?.autoCompile ?? false;
  const autoOpenOutput = settings?.assembly?.autoOpenOutput ?? false;

  // Initialize WASM assembler on mount
  useEffect(() => {
    initAssembler().catch(console.error);
  }, []);

  // Subscribe to tokenizer state changes and trigger retokenization
  useEffect(() => {
    if (!assemblyEditor) return;

    const subscription = assemblyTokenizerState$.subscribe((state) => {
      if (state.initialized) {
        console.log(
          'Assembly tokenizer updated with WASM instructions, triggering retokenization',
        );
        // Force retokenization by triggering a state update
        const currentState = assemblyEditor.editorState.value;
        assemblyEditor.editorState.next({
          ...currentState,
          lines: [...currentState.lines], // Create new array reference to trigger update
        });
      }
    });

    return () => subscription.unsubscribe();
  }, [assemblyEditor]);

  // Subscribe to minimap state
  const [minimapEnabled, setMinimapEnabled] = useLocalStorageState(
    'assemblyMinimap',
    false,
  );

  // Create quick nav store
  const quickNavStore = useMemo(() => new AssemblyQuickNavStore(), []);

  // Subscribe to assembly output and automatically send to macro editor
  const outputState = useStoreSubscribe(assemblyOutputStore.state);
  useEffect(() => {
    if (outputState.output && !outputState.error) {
      assemblyToMacroService.processAssemblyOutput(
        outputState.output.instructions,
        outputState.output.memoryData,
      );
    }
  }, [outputState.output, outputState.error]);

  useEffect(() => {
    if (assemblyEditor) {
      const sub = assemblyEditor.showMinimap.subscribe(setMinimapEnabled);
      return () => sub.unsubscribe();
    }
  }, [assemblyEditor, setMinimapEnabled]);

  // Extract navigation items (labels and marks) from the code
  const extractNavigationItems = useCallback((): AssemblyNavigationItem[] => {
    if (!assemblyEditor) return [];

    const lines = assemblyEditor.editorState.getValue().lines;
    const items: AssemblyNavigationItem[] = [];

    lines.forEach((line, lineIndex) => {
      // Extract labels
      const labelMatch = line.text.match(/^([a-zA-Z_][a-zA-Z0-9_]*):/);
      if (labelMatch) {
        items.push({
          type: 'label',
          name: labelMatch[1],
          line: lineIndex,
          column: 0,
        });
      }

      // Extract mark comments (// MARK:)
      const markMatch = line.text.match(/\/\/\s*MARK:\s*(.+)/);
      if (markMatch) {
        items.push({
          type: 'mark',
          name: markMatch[1].trim(),
          line: lineIndex,
          column: line.text.indexOf('// MARK:'),
        });
      }
    });

    return items;
  }, [assemblyEditor]);

  // Handle keyboard shortcuts
  useEffect(() => {
    if (!assemblyEditor) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+P for quick navigation
      if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
        e.preventDefault();
        const items = extractNavigationItems();
        quickNavStore.show(items);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [assemblyEditor, quickNavStore, extractNavigationItems]);

  useEffect(() => {
    // Update minimap setting when it changes
    if (assemblyEditor) {
      assemblyEditor.showMinimap.next(minimapEnabled);
    }
  }, [minimapEnabled, assemblyEditor]);

  // Function to assemble code
  const assembleCode = useCallback(async () => {
    if (!assemblyEditor) return;

    const code = assemblyEditor.getText();

    try {
      // Use Rust WASM assembler
      const assembler = createAssembler({
        bankSize: settings?.assembly?.bankSize,
        maxImmediate: settings?.assembly?.maxImmediate,
        memoryOffset: settings?.assembly?.memoryOffset,
      });

      const result = await assembler.assemble(code);

      if (result.errors.length > 0) {
        // Report errors
        assemblyOutputStore.setError(result.errors.join('\n'));

        // TODO: Add inline error reporting
        console.error('Assembly errors:', result.errors);
      } else {
        // Set successful output
        assemblyOutputStore.setOutput({
          instructions: result.instructions,
          labels: result.labels,
          dataLabels: result.dataLabels,
          memoryData: result.memoryData,
        });

        // Auto-open output panel if configured
        if (autoOpenOutput && !showOutput) {
          setShowOutput(true);
        }
      }
    } catch (error) {
      assemblyOutputStore.setError(`Assembly failed: ${error}`);
      console.error('Assembly error:', error);
    }
  }, [
    assemblyEditor,
    autoOpenOutput,
    showOutput,
    setShowOutput,
    settings?.assembly?.bankSize,
    settings?.assembly?.maxImmediate,
    settings?.assembly?.memoryOffset,
  ]);

  // Auto-compile effect
  useEffect(() => {
    if (!autoCompile || !assemblyEditor) return;

    let timeoutId: number;

    // Subscribe to editor changes
    const subscription = assemblyEditor.editorState.subscribe(() => {
      // Clear previous timeout
      clearTimeout(timeoutId);

      // Debounce the compilation
      timeoutId = setTimeout(() => {
        assembleCode();
      }, 500); // 500ms delay for more responsive feedback
    });

    // Initial compilation
    assembleCode();

    return () => {
      clearTimeout(timeoutId);
      subscription.unsubscribe();
    };
  }, [autoCompile, assemblyEditor, assembleCode]);

  const handleResize = useCallback(
    (leftWidth: number) => {
      const container = document.querySelector('.assembly-editor-container');
      if (container) {
        const containerWidth = container.clientWidth;
        const percentage = (leftWidth / containerWidth) * 100;
        setLeftPanelWidth(Math.max(20, Math.min(80, percentage)));
      }
    },
    [setLeftPanelWidth],
  );

  // Handle jump to label
  const handleJumpToLabel = useCallback(
    (labelName: string) => {
      if (!assemblyEditor) return;

      const lines = assemblyEditor.getText().split('\n');
      for (let i = 0; i < lines.length; i++) {
        const labelMatch = lines[i].match(/^([a-zA-Z_][a-zA-Z0-9_]*):/);
        if (labelMatch && labelMatch[1] === labelName) {
          // Set navigation flag for center scrolling
          assemblyEditor.isNavigating.next(true);
          // Jump to the line with the label
          assemblyEditor.setCursorPosition({ line: i, column: 0 });
          // Focus the editor
          assemblyEditor.focus();
          break;
        }
      }
    },
    [assemblyEditor],
  );

  // Handle quick navigation
  const handleQuickNavigate = useCallback(
    (item: AssemblyNavigationItem) => {
      if (!assemblyEditor) return;

      // Set navigation flag for center scrolling
      assemblyEditor.isNavigating.next(true);
      // Jump to the item
      assemblyEditor.setCursorPosition({
        line: item.line,
        column: item.column,
      });
      // Focus the editor
      assemblyEditor.focus();
    },
    [assemblyEditor],
  );

  if (!assemblyEditor) {
    return <div className="v grow-1 bg-zinc-950">Loading...</div>;
  }

  return (
    <div className="v grow-1 bg-zinc-950">
      <HSep />
      <div className="h grow-1 relative assembly-editor-container">
        <div
          className="v grow-0 shrink-0 bg-zinc-950"
          style={{ width: showOutput ? `${leftPanelWidth}%` : '100%' }}
        >
          <div className="h items-center bg-zinc-900 text-zinc-500 text-xs font-bold p-2 min-h-8 border-b border-zinc-800">
            <span className="mr-4">Assembly Editor</span>

            <div className="w-px h-6 bg-zinc-700 mx-1" />

            <IconButton
              icon={CpuChipIcon}
              label="Assemble"
              onClick={assembleCode}
            />

            <div className="w-px h-6 bg-zinc-700 mx-1" />

            <IconButton
              icon={DocumentTextIcon}
              label="Toggle Minimap"
              onClick={() => {
                const newValue = !minimapEnabled;
                setMinimapEnabled(newValue);
                assemblyEditor?.showMinimap.next(newValue);
              }}
              variant={minimapEnabled ? 'info' : 'default'}
            />

            <div className="w-px h-6 bg-zinc-700 mx-1" />

            <IconButton
              icon={ArrowPathIcon}
              label={autoCompile ? 'Auto-compile On' : 'Auto-compile Off'}
              onClick={() => settingsStore.setAssemblyAutoCompile(!autoCompile)}
              variant={autoCompile ? 'info' : 'default'}
            />

            <div className="ml-auto h gap-2">
              <button
                className="text-zinc-600 hover:text-zinc-400"
                onClick={() => setShowOutput(!showOutput)}
              >
                {showOutput ? 'Hide Output' : 'Show Output'}
              </button>
            </div>
          </div>
          <Editor
            store={assemblyEditor}
            onFocus={() => editorManager.setActiveEditor('assembly')}
          />
        </div>
        {showOutput && (
          <>
            <DraggableVSep onResize={handleResize} />
            <div className="v grow-1 bg-zinc-950">
              <AssemblyOutput onJumpToLabel={handleJumpToLabel} />
            </div>
          </>
        )}
      </div>

      {/* Quick Navigation Modal */}
      <AssemblyQuickNav
        quickNavStore={quickNavStore}
        onNavigate={handleQuickNavigate}
      />
    </div>
  );
}
