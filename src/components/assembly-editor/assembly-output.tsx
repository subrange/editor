import { useStoreSubscribe } from '../../hooks/use-store-subscribe.tsx';
import { assemblyOutputStore } from '../../stores/assembly-output.store.ts';
import { useState, useCallback } from 'react';
import clsx from 'clsx';
import {
  ClipboardDocumentIcon,
  ArrowDownTrayIcon,
} from '@heroicons/react/24/outline';
import { formatMacro } from '../../services/ripple-assembler/index.ts';
import { useLocalStorageState } from '../../hooks/use-local-storage-state.tsx';
import { BfMacroDisplay } from './components/bf-macro-display.tsx';

interface AssemblyOutputProps {
  onJumpToLabel?: (label: string) => void;
}

export function AssemblyOutput({ onJumpToLabel }: AssemblyOutputProps) {
  const outputState = useStoreSubscribe(assemblyOutputStore.state);
  const [selectedTab, setSelectedTab] = useLocalStorageState<
    'assembly' | 'labels' | 'data' | 'macros'
  >('assemblySelectedTab', 'macros');
  const [showCopied, setShowCopied] = useState(false);

  const { output, error, isCompiling } = outputState;

  const copyToClipboard = useCallback(() => {
    if (!output) return;

    let content = '';
    if (selectedTab === 'assembly') {
      // Format instructions as hex
      content = output.instructions
        .map((instr, i) => {
          const hex = [
            instr.opcode,
            instr.word0,
            instr.word1,
            instr.word2,
            instr.word3,
          ]
            .map((b) => b.toString(16).padStart(2, '0').toUpperCase())
            .join(' ');
          return `${i.toString().padStart(4, '0')}: ${hex}`;
        })
        .join('\n');
    } else if (selectedTab === 'macros') {
      // Format as Brainfuck macros
      content = formatMacro(output.instructions, output.memoryData);
    }

    navigator.clipboard.writeText(content);

    // Show copied indicator
    setShowCopied(true);
    setTimeout(() => setShowCopied(false), 2000);
  }, [output, selectedTab]);

  const downloadAsFile = useCallback(() => {
    if (!output) return;

    let content = '';
    let filename = '';

    if (selectedTab === 'assembly') {
      // Binary format
      const bytes = output.instructions.flatMap((instr) => [
        instr.opcode,
        instr.word0,
        instr.word1,
        instr.word2,
        instr.word3,
      ]);
      const blob = new Blob([new Uint8Array(bytes)], {
        type: 'application/octet-stream',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'output.bin';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      return;
    } else if (selectedTab === 'macros') {
      content = formatMacro(output.instructions, output.memoryData);
      filename = 'output.bfm';
    }

    if (content) {
      const blob = new Blob([content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  }, [output, selectedTab]);

  const tabClasses = (isActive: boolean) =>
    clsx(
      'px-3 py-1.5 text-xs font-medium transition-colors',
      isActive
        ? 'text-zinc-300 bg-zinc-800 border-b-2 border-blue-500'
        : 'text-zinc-500 hover:text-zinc-400 hover:bg-zinc-800/50',
    );

  return (
    <div className="v h-full">
      <div className="h items-center bg-zinc-900 text-zinc-500 text-xs font-bold px-2 min-h-8 border-b border-zinc-800">
        <span>Assembly Output</span>

        {/* Tabs */}
        <div className="h gap-0 ml-4">
          <button
            className={tabClasses(selectedTab === 'macros')}
            onClick={() => setSelectedTab('macros')}
          >
            BF Output
          </button>
          <button
            className={tabClasses(selectedTab === 'assembly')}
            onClick={() => setSelectedTab('assembly')}
          >
            Assembly
          </button>
          <button
            className={tabClasses(selectedTab === 'labels')}
            onClick={() => setSelectedTab('labels')}
          >
            Labels
          </button>
          <button
            className={tabClasses(selectedTab === 'data')}
            onClick={() => setSelectedTab('data')}
          >
            Data
          </button>
        </div>

        {/* Actions */}
        <div className="ml-auto h gap-2">
          {output && (
            <>
              <button
                onClick={copyToClipboard}
                className={clsx(
                  'p-1 transition-all duration-200 relative',
                  showCopied
                    ? 'text-green-400'
                    : 'text-zinc-600 hover:text-zinc-400',
                )}
                title={showCopied ? 'Copied!' : 'Copy to clipboard'}
              >
                <ClipboardDocumentIcon
                  className={clsx(
                    'w-4 h-4 transition-transform duration-200',
                    showCopied && 'scale-110',
                  )}
                />
                {showCopied && (
                  <span className="absolute -top-8 left-1/2 -translate-x-1/2 bg-green-500 text-white text-xs px-2 py-1 rounded whitespace-nowrap">
                    Copied!
                  </span>
                )}
              </button>
              <button
                onClick={downloadAsFile}
                className="p-1 text-zinc-600 hover:text-zinc-400 transition-colors"
                title="Download"
              >
                <ArrowDownTrayIcon className="w-4 h-4" />
              </button>
            </>
          )}
        </div>
      </div>

      <div className="grow-1 overflow-auto p-4 bg-zinc-950">
        {isCompiling && (
          <div className="text-zinc-500 text-sm">Compiling...</div>
        )}

        {error && (
          <div className="text-red-400 text-sm font-mono whitespace-pre">
            {error}
          </div>
        )}

        {output && !error && (
          <>
            {selectedTab === 'assembly' && (
              <div className="font-mono text-xs">
                <div className="text-zinc-500 mb-2">
                  {output.instructions.length} instructions (
                  {output.instructions.length * 5} bytes)
                </div>
                <div className="space-y-1">
                  {output.instructions.map((instr, i) => {
                    const hex = [
                      instr.opcode,
                      instr.word0,
                      instr.word1,
                      instr.word2,
                      instr.word3,
                    ]
                      .map((b) => b.toString(16).padStart(2, '0').toUpperCase())
                      .join(' ');
                    return (
                      <div key={i} className="text-zinc-300">
                        <span className="text-zinc-600">
                          {i.toString().padStart(4, '0')}:
                        </span>{' '}
                        <span className="text-blue-400">{hex}</span>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}

            {selectedTab === 'labels' && (
              <div className="font-mono text-xs space-y-1">
                {Array.from(output.labels.entries()).map(([name, label]) => (
                  <div
                    key={name}
                    className="text-zinc-300 hover:text-zinc-100 cursor-pointer"
                    onClick={() => onJumpToLabel?.(name)}
                  >
                    <span className="text-rose-400">{name}:</span>{' '}
                    <span className="text-zinc-500">
                      Bank {label.bank}, Offset {label.offset} (Addr:{' '}
                      {label.absoluteAddress})
                    </span>
                  </div>
                ))}
                {output.labels.size === 0 && (
                  <div className="text-zinc-600">No labels defined</div>
                )}
              </div>
            )}

            {selectedTab === 'data' && (
              <div className="font-mono text-xs">
                <div className="text-zinc-500 mb-2">
                  Data section: {output.memoryData.length} bytes
                </div>
                {output.dataLabels.size > 0 && (
                  <div className="space-y-2 mb-4">
                    {Array.from(output.dataLabels.entries()).map(
                      ([name, offset]) => (
                        <div key={name} className="text-zinc-300">
                          <span className="text-purple-400">{name}:</span>{' '}
                          <span className="text-zinc-500">Offset {offset}</span>
                        </div>
                      ),
                    )}
                  </div>
                )}
                <div className="grid grid-cols-16 gap-1">
                  {output.memoryData.map((byte, i) => (
                    <div
                      key={i}
                      className="text-zinc-400 text-center"
                      title={`Offset ${i}: ${byte} (0x${byte.toString(16).toUpperCase()})`}
                    >
                      {byte.toString(16).padStart(2, '0').toUpperCase()}
                    </div>
                  ))}
                </div>
                {output.memoryData.length === 0 && (
                  <div className="text-zinc-600">No data defined</div>
                )}
              </div>
            )}

            {selectedTab === 'macros' && (
              <BfMacroDisplay
                content={formatMacro(output.instructions, output.memoryData)}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
}
