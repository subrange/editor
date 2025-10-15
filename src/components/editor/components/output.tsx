import {
  useStoreSubscribeToField,
  useStoreSubscribe,
} from '../../../hooks/use-store-subscribe.tsx';
import { interpreterStore } from '../../debugger/interpreter-facade.store.ts';
import { useLayoutEffect, useRef, useEffect } from 'react';
import clsx from 'clsx';
import {
  ChevronDownIcon,
  ChevronUpIcon,
  XMarkIcon,
} from '@heroicons/react/16/solid';
import { CommandLineIcon } from '@heroicons/react/24/outline';
import { outputStore } from '../../../stores/output.store.ts';
import { VMOutput } from './vm-output.tsx';
import { Disassembly } from './disassembly.tsx';
import { IO } from './io.tsx';
import { useLocalStorageState } from '../../../hooks/use-local-storage-state.tsx';
import { settingsStore } from '../../../stores/settings.store.ts';

interface OutputProps {
  position?: 'bottom' | 'right' | 'floating';
  showHeader?: boolean;
  onClose?: () => void;
}

export function Output({
  position = 'bottom',
  showHeader = true,
  onClose,
}: OutputProps) {
  const [activeTab, setActiveTab] = useLocalStorageState<
    'output' | 'vm' | 'disassembly'
  >('output-panel-active-tab', 'output');
  const [splitView, setSplitView] = useLocalStorageState<boolean>(
    'output-panel-split-view',
    false,
  );
  const outputState = useStoreSubscribe(outputStore.state);
  const output = useStoreSubscribeToField(interpreterStore.state, 'output');
  const interpreterState = useStoreSubscribe(interpreterStore.state);
  const settings = useStoreSubscribe(settingsStore.settings);
  const outputContainer = useRef<HTMLDivElement>(null);
  const vmOutputContainer = useRef<HTMLDivElement>(null);

  const showAssemblyWorkspace = settings?.assembly?.showWorkspace ?? false;
  const showDisassembly =
    (settings?.debugger.showDisassembly ?? false) && showAssemblyWorkspace;
  const isDebugging = interpreterState.isRunning || interpreterState.isPaused;

  const { collapsed, height, maxLines } = outputState;

  // Switch to output tab if current tab is not available or if input is needed
  useEffect(() => {
    if (
      !showAssemblyWorkspace &&
      (activeTab === 'vm' || activeTab === 'disassembly')
    ) {
      setActiveTab('output');
    }

    // Switch to output tab and expand panel when input is needed
    if (interpreterState.isWaitingForInput) {
      if (activeTab !== 'output') {
        console.log('Output: Switching to output tab for input');
        setActiveTab('output');
      }
      if (collapsed) {
        console.log('Output: Expanding panel for input');
        outputStore.setCollapsed(false);
      }
    }
  }, [
    showAssemblyWorkspace,
    activeTab,
    setActiveTab,
    interpreterState.isWaitingForInput,
    collapsed,
  ]);

  // Note: Auto-scroll and output processing are now handled by individual components (IO, VMOutput, Disassembly)

  // Auto-enable split view when debugging starts (optional feature)
  useLayoutEffect(() => {
    if (isDebugging && showDisassembly && !splitView) {
      // Optionally auto-enable split view when debugging starts
      // Uncomment the line below to enable this feature:
      // setSplitView(true);
    }
  }, [isDebugging, showDisassembly, splitView]);

  const containerClasses = clsx(
    'v bg-zinc-900 transition-all overflow-hidden',
    {
      // Bottom position styles
      'h-96 min-h-96': position === 'bottom' && !collapsed,
      'h-8 min-h-8': position === 'bottom' && collapsed,

      // Right position styles
      'h-full grow-1': position === 'right' && !collapsed,
      'w-8 min-w-8 h-full': position === 'right' && collapsed,

      // Floating position styles
      'absolute bottom-4 right-4 w-96 h-64 shadow-2xl rounded-lg border border-zinc-800':
        position === 'floating',
    },
  );

  const headerClasses = clsx(
    'bg-zinc-900 text-zinc-500 text-xs font-bold h-8 min-h-8 flex-shrink-0',
    {
      h: !collapsed || position !== 'right',
      'v items-center justify-center': collapsed && position === 'right',
      'border-t border-zinc-800': position === 'bottom',
      'border-l border-zinc-800': position === 'right',
      'rounded-t-lg': position === 'floating',
    },
  );

  const tabButtonClasses = (isActive: boolean) =>
    clsx('px-3 py-2 text-xs font-bold transition-colors', {
      'text-zinc-400 bg-zinc-800': isActive,
      'text-zinc-600 hover:text-zinc-500 hover:bg-zinc-800/50': !isActive,
    });

  const contentClasses = clsx('p-2 bg-zinc-950 grow-1 min-h-0', {
    'rounded-b-lg': position === 'floating',
    'flex flex-col': !splitView || !showDisassembly,
  });

  return (
    <div
      className={containerClasses}
      style={{
        height: position === 'bottom' && !collapsed ? height : undefined,
      }}
    >
      {showHeader && (
        <div className={headerClasses}>
          {collapsed ? (
            // When collapsed, show a simple button
            <button
              className="w-full h-full flex items-center justify-center gap-2 hover:bg-zinc-800 transition-colors"
              onClick={() => outputStore.setCollapsed(false)}
            >
              {position === 'right' ? (
                <CommandLineIcon className="w-4 h-4" />
              ) : (
                <>
                  <ChevronUpIcon className="w-4 h-4" />
                  <span>I/O</span>
                </>
              )}
            </button>
          ) : (
            // When expanded, show full header with tabs
            <>
              {/* Collapse button */}
              <button
                className="p-2 hover:bg-zinc-800 transition-colors"
                onClick={() => outputStore.setCollapsed(true)}
                title="Collapse panel"
              >
                {position === 'right' ? (
                  <ChevronUpIcon className="w-4 h-4 rotate-90" />
                ) : (
                  <ChevronDownIcon className="w-4 h-4" />
                )}
              </button>

              {/* Tabs */}
              <div className="h gap-0">
                {!splitView ? (
                  // Normal tab mode
                  <>
                    <button
                      className={tabButtonClasses(activeTab === 'output')}
                      onClick={() => setActiveTab('output')}
                    >
                      I/O
                      {interpreterState.isWaitingForInput && (
                        <span className="ml-1 inline-block w-2 h-2 bg-blue-500 rounded-full animate-pulse" />
                      )}
                    </button>
                    {showAssemblyWorkspace && (
                      <button
                        className={tabButtonClasses(activeTab === 'vm')}
                        onClick={() => setActiveTab('vm')}
                      >
                        VM Output
                      </button>
                    )}
                    {showDisassembly && (
                      <button
                        className={tabButtonClasses(
                          activeTab === 'disassembly',
                        )}
                        onClick={() => setActiveTab('disassembly')}
                      >
                        Disassembly
                      </button>
                    )}
                  </>
                ) : (
                  // Split view mode - fixed layout
                  <>
                    <span className="px-3 py-2 text-xs text-zinc-500">
                      Debug View: Disassembly + VM Output
                    </span>
                  </>
                )}
              </div>

              {/* Additional controls */}
              <div className="ml-auto h gap-2 p-2">
                {/* Split view toggle */}
                {showDisassembly && (
                  <button
                    onClick={() => setSplitView(!splitView)}
                    className={clsx(
                      'px-1.5 text-xs transition-colors',
                      splitView
                        ? 'text-blue-500 hover:text-blue-400'
                        : 'text-zinc-600 hover:text-zinc-400',
                    )}
                    title={
                      splitView
                        ? 'Switch to tabs'
                        : 'Debug view (Disassembly + VM)'
                    }
                  >
                    {splitView ? '◫' : '◱'}
                  </button>
                )}

                {activeTab === 'output' && !splitView && (
                  <button
                    onClick={() => {
                      if (outputContainer.current) {
                        outputContainer.current.scrollTop =
                          outputContainer.current.scrollHeight;
                      }
                    }}
                    className="text-zinc-600 hover:text-zinc-400"
                    title="Scroll to bottom"
                  >
                    ↓
                  </button>
                )}

                {position === 'floating' && (
                  <button
                    onClick={() => onClose?.()}
                    className="text-zinc-600 hover:text-zinc-400"
                  >
                    <XMarkIcon className="w-4 h-4" />
                  </button>
                )}
              </div>
            </>
          )}
        </div>
      )}

      {!collapsed &&
        (splitView && showDisassembly ? (
          // Split view mode - show Disassembly on top, VM Output on bottom
          <div
            className={clsx(
              'flex flex-col gap-2 h-full overflow-hidden p-2 bg-zinc-950',
              position === 'floating' && 'rounded-b-lg',
            )}
          >
            {/* Top panel - Disassembly */}
            <div className="flex-1 flex flex-col min-h-0 border-b border-zinc-800">
              <div
                className="flex-1 overflow-auto min-h-0"
                ref={outputContainer}
              >
                <Disassembly outputRef={outputContainer} isActive={true} />
              </div>
            </div>

            {/* Bottom panel - VM Output */}
            <div className="flex-1 flex flex-col min-h-0 max-h-[200px]">
              <div className="text-zinc-400 text-xs font-bold mb-2 flex-shrink-0">
                VM Output
              </div>
              <div
                className="flex-1 overflow-auto min-h-0"
                ref={vmOutputContainer}
              >
                <VMOutput outputRef={vmOutputContainer} isActive={true} />
              </div>
            </div>
          </div>
        ) : (
          // Tab view mode - show single panel
          <div
            className={clsx(contentClasses, 'overflow-auto')}
            ref={outputContainer}
          >
            {activeTab === 'output' ? (
              <IO
                output={output}
                maxLines={maxLines}
                outputRef={outputContainer}
                isActive={activeTab === 'output'}
              />
            ) : activeTab === 'vm' ? (
              <VMOutput
                outputRef={outputContainer}
                isActive={activeTab === 'vm'}
              />
            ) : (
              <Disassembly
                outputRef={outputContainer}
                isActive={activeTab === 'disassembly'}
              />
            )}
          </div>
        ))}
    </div>
  );
}
