import { useState, useRef, useEffect } from 'react';
import { interpreterStore } from "./interpreter-facade.store.ts";
import { useStoreSubscribe } from "../../hooks/use-store-subscribe.tsx";
import { settingsStore } from "../../stores/settings.store.ts";
import { IconButton } from "../ui/icon-button.tsx";
import { Bars3Icon, Bars2Icon, ViewColumnsIcon, Square2StackIcon, Squares2X2Icon } from '@heroicons/react/24/solid';
import { TapeCanvasRenderer } from './tape-canvas-renderer';
import clsx from "clsx";

export function Debugger() {
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });
  const containerRef = useRef<HTMLDivElement>(null);
  const interpreterState = useStoreSubscribe(interpreterStore.state);
  const settings = useStoreSubscribe(settingsStore.settings);
  const [showGoToCell, setShowGoToCell] = useState(false);
  
  const tape = interpreterState.tape;
  const pointer = interpreterState.pointer;
  const laneCount = interpreterState.laneCount;
  const viewMode = settings?.debugger.viewMode ?? 'normal';
  
  // Determine cell info
  const cellInfo = tape instanceof Uint8Array
    ? { bits: 8, bytes: 1, max: 255 }
    : tape instanceof Uint16Array
      ? { bits: 16, bytes: 2, max: 65535 }
      : tape instanceof Uint32Array
        ? { bits: 32, bytes: 4, max: 4294967295 }
        : { bits: 8, bytes: 1, max: 255 };
  
  // Update container size
  useEffect(() => {
    const updateSize = () => {
      if (containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect();
        setContainerSize({ width: rect.width, height: rect.height });
      }
    };
    
    updateSize();
    window.addEventListener('resize', updateSize);
    
    const resizeObserver = new ResizeObserver(updateSize);
    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }
    
    return () => {
      window.removeEventListener('resize', updateSize);
      resizeObserver.disconnect();
    };
  }, []);
  
  return (
    <div className="flex flex-col h-full bg-zinc-950">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-zinc-800 bg-zinc-900 px-4 py-2">
        <div className="flex items-center gap-4">
          <h3 className="text-sm font-medium text-zinc-300">Memory Tape</h3>
          <div className="flex items-center gap-2 text-xs text-zinc-500">
            <span className="px-2 py-0.5 rounded-sm bg-zinc-800">
              {cellInfo.bits}-bit cells
            </span>
            <span>•</span>
            <span>Pointer: {pointer}</span>
            <span>•</span>
            <span>Value: {tape[pointer]}</span>
            {viewMode === 'lane' && laneCount > 1 && (
              <>
                <span>•</span>
                <span>Words: {Math.ceil(tape.length / laneCount)}</span>
              </>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          {/* View mode toggle buttons */}
          <div className="flex items-center gap-1 border-r border-zinc-700 pr-2 mr-1">
            <IconButton
              icon={Bars3Icon}
              label="Normal View"
              onClick={() => settingsStore.setDebuggerViewMode('normal')}
              variant={viewMode === 'normal' ? 'info' : 'default'}
            />
            <IconButton
              icon={Bars2Icon}
              label="Compact View"
              onClick={() => settingsStore.setDebuggerViewMode('compact')}
              variant={viewMode === 'compact' ? 'info' : 'default'}
            />
            <IconButton
              icon={ViewColumnsIcon}
              label="Lane View"
              onClick={() => settingsStore.setDebuggerViewMode('lane')}
              variant={viewMode === 'lane' ? 'info' : 'default'}
            />
          </div>
          {/* Navigation buttons */}
          <button
            onClick={() => {
              const renderer = containerRef.current?.querySelector('canvas');
              if (renderer) {
                renderer.dispatchEvent(new CustomEvent('scrollToIndex', { detail: { index: 0 } }));
              }
            }}
            className="text-xs px-3 py-1 rounded-sm bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors"
          >
            Go to Start
          </button>
          <button
            onClick={() => {
              const renderer = containerRef.current?.querySelector('canvas');
              if (renderer) {
                renderer.dispatchEvent(new CustomEvent('scrollToIndex', { detail: { index: pointer } }));
              }
            }}
            className="text-xs px-3 py-1 rounded-sm bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors"
          >
            Go to Pointer
          </button>
          <button
            onClick={() => {
              const renderer = containerRef.current?.querySelector('canvas');
              if (renderer) {
                renderer.dispatchEvent(new CustomEvent('scrollToIndex', { detail: { index: tape.length - 1 } }));
              }
            }}
            className="text-xs px-3 py-1 rounded-sm bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors"
          >
            Go to End
          </button>
          <button
            onClick={() => setShowGoToCell(true)}
            className="text-xs px-3 py-1 rounded-sm bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors"
          >
            Go to Cell
          </button>
        </div>
      </div>
      
      {/* Canvas container */}
      <div ref={containerRef} className="flex-1 relative overflow-hidden">
        {containerSize.width > 0 && containerSize.height > 0 && (
          <TapeCanvasRenderer
            width={containerSize.width} 
            height={containerSize.height}
            viewMode={viewMode}
            laneCount={laneCount}
          />
        )}
      </div>
      
      {/* Status bar */}
      <div className="flex items-center justify-between border-t border-zinc-800 bg-zinc-900 px-4 py-2 text-xs text-zinc-500">
        <div className="flex items-center gap-3">
          <span>Memory: {tape.length.toLocaleString()} cells</span>
          <span>•</span>
          <span>Range: 0-{cellInfo.max.toLocaleString()}</span>
        </div>
        <span>Scroll with mouse wheel or trackpad</span>
      </div>
      
      {/* Go to Cell Modal */}
      {showGoToCell && (
        <>
          {/* Backdrop */}
          <div 
            className="fixed inset-0 bg-black/50 z-40" 
            onClick={() => setShowGoToCell(false)}
          />
          
          <div className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl overflow-hidden">
            {/* Header */}
            <div className="px-4 py-3 bg-zinc-800 border-b border-zinc-700">
              <h3 className="text-sm font-medium text-zinc-300">Go to Cell</h3>
              <p className="text-xs text-zinc-500 mt-0.5">
                Enter a cell number or math expression (0-{tape.length - 1})
              </p>
            </div>
            
            {/* Input section */}
            <div className="p-4">
              <input
                type="text"
                placeholder="Cell number or expression (e.g., 16 * 4, 256 + 4)..."
                className="w-full px-3 py-2 text-sm bg-zinc-800 border border-zinc-700 rounded focus:border-zinc-600 focus:outline-none focus:ring-1 focus:ring-zinc-600"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    const input = (e.target as HTMLInputElement).value.trim();
                    let value: number;
                    
                    try {
                      // Try to evaluate as a math expression
                      // Only allow numbers and basic math operators for safety
                      if (/^[\d\s+\-*/().]+$/.test(input)) {
                        value = Math.floor(eval(input));
                      } else {
                        // If not a valid expression, try parsing as number
                        value = parseInt(input, 10);
                      }
                      
                      if (!isNaN(value) && value >= 0 && value < tape.length) {
                        const renderer = containerRef.current?.querySelector('canvas');
                        if (renderer) {
                          renderer.dispatchEvent(new CustomEvent('scrollToIndex', { detail: { index: value } }));
                        }
                        setShowGoToCell(false);
                      } else {
                        // Show error in input
                        (e.target as HTMLInputElement).classList.add('border-red-500');
                        setTimeout(() => {
                          (e.target as HTMLInputElement).classList.remove('border-red-500');
                        }, 1000);
                      }
                    } catch (error) {
                      // Invalid expression - show error
                      (e.target as HTMLInputElement).classList.add('border-red-500');
                      setTimeout(() => {
                        (e.target as HTMLInputElement).classList.remove('border-red-500');
                      }, 1000);
                    }
                  } else if (e.key === 'Escape') {
                    setShowGoToCell(false);
                  }
                }}
              />
              <p className="text-xs text-zinc-500 mt-2">
                Examples: 42, 16*4, 256+4, (128*2)-1 • Press Enter to go, Escape to cancel
              </p>
            </div>
          </div>
        </>
      )}
    </div>
  );
}