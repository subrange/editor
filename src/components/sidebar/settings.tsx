import clsx from 'clsx';
import { interpreterStore } from '../debugger/interpreter-facade.store.ts';
import { settingsStore } from '../../stores/settings.store.ts';
import { outputStore } from '../../stores/output.store.ts';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe.tsx';
import { TapeLabelsEditor } from './tape-labels-editor';
import { ChevronDownIcon, ChevronRightIcon } from '@heroicons/react/24/solid';
import { useState, useRef } from 'react';
import { settingsManager } from '../../services/settings-manager.service';
import { ExclamationTriangleIcon } from '@heroicons/react/24/outline';

function SettingSection({
  title,
  children,
  defaultOpen = false,
}: {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}) {
  const storageKey = `settings-section-${title.toLowerCase().replace(/\s+/g, '-')}`;
  const [isOpen, setIsOpen] = useState(() => {
    const stored = localStorage.getItem(storageKey);
    return stored !== null ? JSON.parse(stored) : defaultOpen;
  });

  const toggleOpen = () => {
    const newState = !isOpen;
    setIsOpen(newState);
    localStorage.setItem(storageKey, JSON.stringify(newState));
  };

  return (
    <div>
      <button
        onClick={toggleOpen}
        className="flex items-center gap-1.5 w-full text-left group py-1.5 px-1 -mx-1 rounded hover:bg-zinc-800/50 transition-colors"
      >
        {isOpen ? (
          <ChevronDownIcon className="w-3.5 h-3.5 text-zinc-500 group-hover:text-zinc-400 transition-colors" />
        ) : (
          <ChevronRightIcon className="w-3.5 h-3.5 text-zinc-500 group-hover:text-zinc-400 transition-colors" />
        )}
        <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider group-hover:text-zinc-300 transition-colors">
          {title}
        </h3>
      </button>
      <div
        className={clsx(
          'overflow-hidden transition-all duration-200',
          isOpen ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0',
        )}
      >
        <div className="mt-3">{children}</div>
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const MAX_TAPE_SIZE = 100_000_000;

export function Settings() {
  const tapeSize = useStoreSubscribe(interpreterStore.tapeSize);
  const cellSize = useStoreSubscribe(interpreterStore.cellSize);
  const laneCount = useStoreSubscribe(interpreterStore.laneCount);
  const settings = useStoreSubscribe(settingsStore.settings);
  const outputState = useStoreSubscribe(outputStore.state);
  const [showResetWarning, setShowResetWarning] = useState(false);

  const handleTapeSizeChange = (value: string) => {
    const size = parseInt(value) || 30000;
    interpreterStore.setTapeSize(Math.max(100, Math.min(MAX_TAPE_SIZE, size)));
  };

  const changeCellSize = (size: number) => {
    interpreterStore.setCellSize(size);
  };

  return (
    <div className="h-full overflow-y-auto w-[268px] border-l border-zinc-800">
      {/* Header */}
      <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-6 py-4 z-10">
        <h2 className="text-lg font-semibold text-zinc-100 whitespace-nowrap">
          Settings
        </h2>
      </div>

      {/* Settings content */}
      <div className="p-6 space-y-4">
        {/* Interpreter Settings */}
        <SettingSection title="Interpreter">
          {/* Tape Size */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium text-zinc-300 whitespace-nowrap">
                Tape Size
              </label>
              <span className="text-xs text-zinc-500 whitespace-nowrap">
                {formatBytes(tapeSize)}
              </span>
            </div>
            <div className="relative">
              <input
                type="range"
                min="100"
                max={MAX_TAPE_SIZE}
                step="100"
                value={tapeSize}
                onChange={(e) => handleTapeSizeChange(e.target.value)}
                className="w-full h-2 bg-zinc-700 rounded appearance-none cursor-pointer slider"
              />
              <input
                type="number"
                value={tapeSize}
                onChange={(e) => handleTapeSizeChange(e.target.value)}
                className="mt-2 w-full px-3 py-2 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                placeholder="Tape size in bytes"
              />
            </div>
          </div>

          {/* Cell Size */}
          <div className="flex flex-col space-y-2 mt-6">
            <label className="text-sm font-medium text-zinc-300 whitespace-nowrap">
              Cell Size
            </label>
            <div className="grid grid-cols-3 gap-2">
              {[
                { value: '256', label: '8-bit', desc: '0-255' },
                { value: '65536', label: '16-bit', desc: '0-65,5K' },
                { value: '4294967296', label: '32-bit', desc: '0-4.3B' },
              ].map((option) => (
                <button
                  key={option.value}
                  onClick={() => changeCellSize(parseInt(option.value, 10))}
                  className={clsx(
                    'p-3 rounded border transition-all text-center',
                    cellSize === parseInt(option.value)
                      ? 'bg-blue-500/20 border-blue-500 text-blue-400'
                      : 'bg-zinc-800 border-zinc-700 text-zinc-300 hover:bg-zinc-700 hover:border-zinc-600',
                  )}
                >
                  <div className="font-medium text-sm">{option.label}</div>
                  <div className="text-[10px] text-zinc-500 mt-1">
                    {option.desc}
                  </div>
                </button>
              ))}
            </div>
          </div>
        </SettingSection>

        {/* Rust WASM Interpreter Settings */}
        <SettingSection title="Rust WASM Interpreter">
          <div className="space-y-4">
            <div className="p-2 bg-zinc-800/50 rounded text-xs text-zinc-400 mb-4">
              <span className="text-yellow-500">⚡</span> These settings only
              apply to the Rust WASM interpreter (Rocket button in toolbar)
            </div>

            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Cell Value Wrapping
              </span>
              <input
                type="checkbox"
                checked={settings?.interpreter?.wrapCells ?? true}
                onChange={(e) =>
                  settingsStore.setInterpreterWrapCells(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Wrap cell values on overflow (255+1→0)
            </p>

            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Tape Pointer Wrapping
              </span>
              <input
                type="checkbox"
                checked={settings?.interpreter?.wrapTape ?? true}
                onChange={(e) =>
                  settingsStore.setInterpreterWrapTape(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Wrap tape pointer at boundaries
            </p>

            <div className="mt-4 pt-4 border-t border-zinc-700">
              <p className="text-xs text-zinc-500">
                <span className="text-zinc-400">Note:</span> The Rust WASM
                interpreter always runs with optimizations enabled and uses the
                tape size and cell size from the main interpreter settings
                above.
              </p>
            </div>
          </div>
        </SettingSection>

        {/* Debugger Settings */}
        <SettingSection title="Debugger">
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium text-zinc-300">
                View Mode
              </label>
              <div className="space-y-2">
                {[
                  {
                    value: 'normal' as const,
                    label: 'Normal',
                    desc: 'Standard horizontal tape view',
                  },
                  {
                    value: 'compact' as const,
                    label: 'Compact',
                    desc: 'Condensed horizontal view',
                  },
                  {
                    value: 'lane' as const,
                    label: 'Lane',
                    desc: 'Lane-based columns (multi-lane only)',
                  },
                ].map((option) => (
                  <button
                    key={option.value}
                    onClick={() =>
                      settingsStore.setDebuggerViewMode(option.value)
                    }
                    className={clsx(
                      'w-full p-2 rounded border transition-all text-left',
                      settings?.debugger.viewMode === option.value
                        ? 'bg-blue-500/20 border-blue-500 text-blue-400'
                        : 'bg-zinc-800 border-zinc-700 text-zinc-300 hover:bg-zinc-700 hover:border-zinc-600',
                    )}
                    disabled={option.value === 'lane' && laneCount === 1}
                  >
                    <div className="font-medium text-sm">{option.label}</div>
                    <div className="text-[10px] text-zinc-500 mt-0.5">
                      {option.desc}
                      {option.value === 'lane' &&
                        laneCount === 1 &&
                        ' (requires lanes > 1)'}
                    </div>
                  </button>
                ))}
              </div>
            </div>

            {/* Lane Count */}
            <div className="space-y-2 mt-4">
              <label className="text-sm font-medium text-zinc-300">
                Lane Count
              </label>
              <div className="flex items-center gap-2">
                <input
                  type="range"
                  min="1"
                  max="10"
                  value={laneCount}
                  onChange={(e) => {
                    const value = Number(e.target.value);
                    interpreterStore.setLaneCount(value);
                    // Automatically switch to lane view when lanes > 1
                    if (value > 1 && settings?.debugger.viewMode !== 'lane') {
                      settingsStore.setDebuggerViewMode('lane');
                    }
                    // Switch back to normal view when lanes = 1
                    else if (
                      value === 1 &&
                      settings?.debugger.viewMode === 'lane'
                    ) {
                      settingsStore.setDebuggerViewMode('normal');
                    }
                  }}
                  className="flex-1 h-2 bg-zinc-800 rounded-lg appearance-none cursor-pointer
                                             [&::-webkit-slider-thumb]:appearance-none
                                             [&::-webkit-slider-thumb]:w-4
                                             [&::-webkit-slider-thumb]:h-4
                                             [&::-webkit-slider-thumb]:rounded-full
                                             [&::-webkit-slider-thumb]:bg-zinc-400
                                             [&::-webkit-slider-thumb]:cursor-pointer
                                             [&::-webkit-slider-thumb]:transition-colors
                                             [&::-webkit-slider-thumb]:hover:bg-zinc-300"
                />
                <input
                  type="number"
                  min="1"
                  max="10"
                  value={laneCount}
                  onChange={(e) => {
                    const value = Number(e.target.value);
                    if (value >= 1 && value <= 10) {
                      interpreterStore.setLaneCount(value);
                      // Automatically switch to lane view when lanes > 1
                      if (value > 1 && settings?.debugger.viewMode !== 'lane') {
                        settingsStore.setDebuggerViewMode('lane');
                      }
                      // Switch back to normal view when lanes = 1
                      else if (
                        value === 1 &&
                        settings?.debugger.viewMode === 'lane'
                      ) {
                        settingsStore.setDebuggerViewMode('normal');
                      }
                    }
                  }}
                  className="w-16 px-2 py-1 text-sm bg-zinc-800 border border-zinc-700 rounded
                                             text-zinc-300 text-center focus:outline-none focus:border-zinc-600"
                />
              </div>
              <p className="text-xs text-zinc-500">
                Visualize tape as interleaved lanes ({laneCount}{' '}
                {laneCount === 1 ? 'lane' : 'lanes'})
              </p>
            </div>
          </div>
        </SettingSection>

        {/* Tape Labels */}
        <SettingSection title="Tape Labels">
          <TapeLabelsEditor />
        </SettingSection>

        {/* Macro Settings */}
        <SettingSection title="Macro Expansion">
          <div className="space-y-4">
            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Auto-expand
              </span>
              <input
                type="checkbox"
                checked={settings?.macro.autoExpand ?? false}
                onChange={(e) =>
                  settingsStore.setMacroAutoExpand(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Automatically expand macros as you type
            </p>

            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Strip Comments
              </span>
              <input
                type="checkbox"
                checked={settings?.macro.stripComments ?? true}
                onChange={(e) =>
                  settingsStore.setMacroStripComments(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Remove all non-Brainfuck characters from expanded code
            </p>

            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Collapse Empty Lines
              </span>
              <input
                type="checkbox"
                checked={settings?.macro.collapseEmptyLines ?? true}
                onChange={(e) =>
                  settingsStore.setMacroCollapseEmptyLines(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Remove lines that contain no Brainfuck commands
            </p>

            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Use Rust WASM Expander
              </span>
              <input
                type="checkbox"
                checked={settings?.macro.useWasmExpander ?? false}
                onChange={(e) =>
                  settingsStore.setMacroUseWasmExpander(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Use Rust-based WASM macro expander
            </p>
          </div>
        </SettingSection>

        {/* Output Settings */}
        <SettingSection title="Output Panel">
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium text-zinc-300">
                Position
              </label>
              <div className="grid grid-cols-2 gap-2">
                {[
                  {
                    value: 'bottom' as const,
                    label: 'Bottom',
                    desc: 'Below editor',
                  },
                  {
                    value: 'right' as const,
                    label: 'Right',
                    desc: 'Side panel',
                  },
                ].map((option) => (
                  <button
                    key={option.value}
                    onClick={() => outputStore.setPosition(option.value)}
                    className={clsx(
                      'p-3 rounded border transition-all text-center',
                      outputState.position === option.value
                        ? 'bg-blue-500/20 border-blue-500 text-blue-400'
                        : 'bg-zinc-800 border-zinc-700 text-zinc-300 hover:bg-zinc-700 hover:border-zinc-600',
                    )}
                  >
                    <div className="font-medium text-sm">{option.label}</div>
                    <div className="text-[10px] text-zinc-500 mt-1">
                      {option.desc}
                    </div>
                  </button>
                ))}
              </div>
            </div>

            {/* Max Lines Setting */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-zinc-300">
                Max Lines
              </label>
              <input
                type="number"
                value={outputState.maxLines || ''}
                onChange={(e) => {
                  const value = e.target.value
                    ? parseInt(e.target.value)
                    : undefined;
                  outputStore.setMaxLines(value);
                }}
                className="w-full px-3 py-2 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                placeholder="No limit"
                min="100"
              />
              <p className="text-xs text-zinc-500">
                Keep only the last N lines (empty for no limit)
              </p>
            </div>
          </div>
        </SettingSection>

        {/* Editor Settings */}
        {/*<SettingSection title="Editor">*/}
        {/*    <div className="space-y-4">*/}
        {/*        <label className="flex items-center justify-between cursor-pointer">*/}
        {/*            <span className="text-sm font-medium text-zinc-300">Syntax highlighting</span>*/}
        {/*            <input*/}
        {/*                type="checkbox"*/}
        {/*                defaultChecked*/}
        {/*                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2"*/}
        {/*            />*/}
        {/*        </label>*/}

        {/*        <label className="flex items-center justify-between cursor-pointer">*/}
        {/*            <span className="text-sm font-medium text-zinc-300">Bracket matching</span>*/}
        {/*            <input*/}
        {/*                type="checkbox"*/}
        {/*                defaultChecked*/}
        {/*                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2"*/}
        {/*            />*/}
        {/*        </label>*/}
        {/*    </div>*/}
        {/*</SettingSection>*/}

        {/* Debug Settings */}
        {/*<SettingSection title="Debugger">*/}
        {/*    <div className="space-y-4">*/}
        {/*        <label className="flex items-center justify-between cursor-pointer">*/}
        {/*            <span className="text-sm font-medium text-zinc-300">Show execution marker</span>*/}
        {/*            <input*/}
        {/*                type="checkbox"*/}
        {/*                defaultChecked*/}
        {/*                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2"*/}
        {/*            />*/}
        {/*        </label>*/}

        {/*        <label className="flex items-center justify-between cursor-pointer">*/}
        {/*            <span className="text-sm font-medium text-zinc-300">Auto-scroll to pointer</span>*/}
        {/*            <input*/}
        {/*                type="checkbox"*/}
        {/*                defaultChecked*/}
        {/*                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2"*/}
        {/*            />*/}
        {/*        </label>*/}
        {/*    </div>*/}
        {/*</SettingSection>*/}

        {/* Assembly Settings */}
        <SettingSection title="Assembly">
          <div className="space-y-4">
            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Show Assembly Workspace
              </span>
              <input
                type="checkbox"
                checked={settings?.assembly?.showWorkspace ?? false}
                onChange={(e) =>
                  settingsStore.setAssemblyShowWorkspace(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Show Assembly workspace tab in the main editor
            </p>

            {/* Show rest of assembly settings only when workspace is enabled */}
            {(settings?.assembly?.showWorkspace ?? false) && (
              <>
                {/* Show Disassembly toggle */}
                <label className="flex items-center justify-between cursor-pointer group mt-4">
                  <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                    Show Disassembly
                  </span>
                  <input
                    type="checkbox"
                    checked={settings?.debugger.showDisassembly ?? false}
                    onChange={(e) =>
                      settingsStore.setDebuggerShowDisassembly(e.target.checked)
                    }
                    className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
                    disabled={
                      settings?.debugger.viewMode !== 'lane' || laneCount === 1
                    }
                  />
                </label>
                <p className="text-xs text-zinc-500 -mt-2">
                  Display disassembly in lane view debugger (requires lane view
                  and lanes &gt; 1)
                </p>

                <label className="flex items-center justify-between cursor-pointer group">
                  <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                    Auto-compile
                  </span>
                  <input
                    type="checkbox"
                    checked={settings?.assembly?.autoCompile ?? false}
                    onChange={(e) =>
                      settingsStore.setAssemblyAutoCompile(e.target.checked)
                    }
                    className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
                  />
                </label>
                <p className="text-xs text-zinc-500 -mt-2">
                  Automatically compile assembly code as you type
                </p>

                <label className="flex items-center justify-between cursor-pointer group">
                  <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                    Auto-open Output
                  </span>
                  <input
                    type="checkbox"
                    checked={settings?.assembly?.autoOpenOutput ?? true}
                    onChange={(e) =>
                      settingsStore.setAssemblyAutoOpenOutput(e.target.checked)
                    }
                    className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
                  />
                </label>
                <p className="text-xs text-zinc-500 -mt-2">
                  Automatically open output panel when compiling
                </p>

                {/* Bank Size */}
                <div className="space-y-2 mt-4">
                  <div className="flex items-center justify-between">
                    <label className="text-sm font-medium text-zinc-300">
                      Bank Size
                    </label>
                    <span className="text-xs text-zinc-500">
                      {settings?.assembly?.bankSize ?? 16} instructions
                    </span>
                  </div>
                  <input
                    type="range"
                    min="4"
                    max="65536"
                    step="4"
                    value={settings?.assembly?.bankSize ?? 16}
                    onChange={(e) =>
                      settingsStore.setAssemblyBankSize(
                        parseInt(e.target.value),
                      )
                    }
                    className="w-full h-2 bg-zinc-700 rounded appearance-none cursor-pointer slider"
                  />
                  <input
                    type="number"
                    min="4"
                    max="64"
                    value={settings?.assembly?.bankSize ?? 16}
                    onChange={(e) => {
                      const value = parseInt(e.target.value) || 16;
                      settingsStore.setAssemblyBankSize(
                        Math.max(4, Math.min(64, value)),
                      );
                    }}
                    className="w-full px-3 py-2 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                    placeholder="Bank size"
                  />
                  <p className="text-xs text-zinc-500">
                    Instructions per memory bank (4-64)
                  </p>
                </div>

                {/* Max Immediate */}
                <div className="space-y-2 mt-4">
                  <label className="text-sm font-medium text-zinc-300">
                    Max Immediate Value
                  </label>
                  <div className="grid grid-cols-3 gap-2">
                    {[
                      { value: 255, label: '8-bit', desc: '0-255' },
                      { value: 65535, label: '16-bit', desc: '0-65,535' },
                      { value: 16777215, label: '24-bit', desc: '0-16.7M' },
                    ].map((option) => (
                      <button
                        key={option.value}
                        onClick={() =>
                          settingsStore.setAssemblyMaxImmediate(option.value)
                        }
                        className={clsx(
                          'p-3 rounded border transition-all text-center',
                          settings?.assembly?.maxImmediate === option.value
                            ? 'bg-blue-500/20 border-blue-500 text-blue-400'
                            : 'bg-zinc-800 border-zinc-700 text-zinc-300 hover:bg-zinc-700 hover:border-zinc-600',
                        )}
                      >
                        <div className="font-medium text-sm">
                          {option.label}
                        </div>
                        <div className="text-[10px] text-zinc-500 mt-1">
                          {option.desc}
                        </div>
                      </button>
                    ))}
                  </div>
                  <p className="text-xs text-zinc-500 mt-2">
                    Maximum value for immediate operands in LI instruction
                  </p>
                </div>
              </>
            )}
          </div>
        </SettingSection>

        {/* Weird Settings */}
        <SettingSection title="Weird">
          <div className="space-y-4">
            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Double Plus Mode
              </span>
              <input
                type="checkbox"
                checked={settings?.weird?.doublePlus ?? false}
                onChange={(e) =>
                  settingsStore.setWeirdDoublePlus(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Make the + instruction add 2 instead of 1
            </p>
          </div>
        </SettingSection>

        {/* IDE Development */}
        <SettingSection title="IDE Development">
          <div className="space-y-4">
            <label className="flex items-center justify-between cursor-pointer group">
              <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                Show Dev Tools
              </span>
              <input
                type="checkbox"
                checked={settings?.development?.showDevTools ?? false}
                onChange={(e) =>
                  settingsStore.setDevelopmentShowDevTools(e.target.checked)
                }
                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
              />
            </label>
            <p className="text-xs text-zinc-500 -mt-2">
              Enable development tools like the learning content capture button
            </p>
          </div>
        </SettingSection>

        {/* Settings Management */}
        <SettingSection title="Settings Management">
          <div className="space-y-4">
            <div className="space-y-2">
              <p className="text-xs text-zinc-500">
                Export or import all IDE settings including files, snapshots,
                and preferences.
              </p>
            </div>

            <div className="space-y-2">
              <button
                onClick={() => settingsManager.downloadSettingsAsFile()}
                className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded transition-colors"
              >
                Export Settings to File
              </button>

              <div className="relative">
                <input
                  type="file"
                  accept=".json"
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (file) {
                      settingsManager
                        .importSettingsFromFile(file)
                        .then(() => {
                          alert('Settings imported successfully!');
                          window.location.reload();
                        })
                        .catch((error) => {
                          alert(`Failed to import settings: ${error.message}`);
                        });
                    }
                    e.target.value = '';
                  }}
                  className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                />
                <button className="w-full px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-zinc-200 text-sm font-medium rounded transition-colors">
                  Import Settings from File
                </button>
              </div>
            </div>

            <div className="pt-2 border-t border-zinc-800">
              <p className="text-xs text-yellow-600">
                ⚠️ Importing settings will overwrite all current settings and
                reload the page.
              </p>
            </div>

            {/* Restore default settings */}
            <div className="pt-4 border-t border-zinc-800">
              <button
                onClick={() => setShowResetWarning(true)}
                className="w-full px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-zinc-200 text-sm font-medium rounded transition-colors"
              >
                Restore Default Settings
              </button>
              <p className="text-xs text-zinc-500 mt-2">
                Reset all settings to their default values
              </p>
            </div>
          </div>
        </SettingSection>
      </div>

      {/* Reset Warning Modal */}
      {showResetWarning && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 bg-black/70 z-40"
            onClick={() => setShowResetWarning(false)}
          />

          {/* Modal */}
          <div className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-md">
            <div className="bg-zinc-900 border border-zinc-700 rounded-lg shadow-2xl overflow-hidden">
              {/* Header */}
              <div className="px-6 py-4 bg-zinc-800 border-b border-zinc-700">
                <div className="flex items-center gap-3">
                  <ExclamationTriangleIcon className="w-6 h-6 text-red-500" />
                  <h3 className="text-lg font-semibold text-zinc-100">
                    Reset All Settings
                  </h3>
                </div>
              </div>

              {/* Content */}
              <div className="px-6 py-4">
                <p className="text-sm text-zinc-300 mb-4">
                  This action will completely reset the IDE to its default
                  state. This includes:
                </p>
                <ul className="list-disc list-inside text-sm text-zinc-400 space-y-1 mb-4">
                  <li>All saved files will be deleted</li>
                  <li>All settings will be reset to defaults</li>
                  <li>All tape snapshots will be removed</li>
                  <li>Editor contents will be cleared</li>
                  <li>Custom tape labels will be removed</li>
                </ul>
                <p className="text-sm text-red-400 ">
                  This action cannot be undone!
                </p>
              </div>

              {/* Actions */}
              <div className="px-6 py-4 bg-zinc-800/50 border-t border-zinc-700 flex justify-end gap-3">
                <button
                  onClick={() => setShowResetWarning(false)}
                  className="px-4 py-2 text-sm font-medium text-zinc-300 bg-zinc-700 hover:bg-zinc-600 rounded transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={() => {
                    // Clear all localStorage
                    localStorage.clear();
                    // Reload the page
                    window.location.reload();
                  }}
                  className="px-4 py-2 text-sm font-medium text-zinc-200 bg-red-900 hover:bg-red-700 rounded transition-colors"
                >
                  Reset Everything
                </button>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
