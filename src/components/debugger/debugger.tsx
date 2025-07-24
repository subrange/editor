import {interpreterStore} from "./interpreter.store.ts";
import {useStoreSubscribe} from "../../hooks/use-store-subscribe.tsx";
import {useVirtualizer} from '@tanstack/react-virtual';
import {useRef, useEffect} from 'react';
import clsx from "clsx";

function formatBinary(value: number, bits: number): string {
    const binary = value.toString(2).padStart(bits, '0');
    // Split into groups of 8 for readability
    if (bits > 8) {
        const groups = [];
        for (let i = binary.length; i > 0; i -= 8) {
            groups.unshift(binary.substring(Math.max(0, i - 8), i));
        }
        return groups.join(' ');
    }
    return binary;
}

function formatHex(value: number, bytes: number): string {
    return '0x' + value.toString(16).padStart(bytes * 2, '0').toUpperCase();
}

function Tape() {
    const interpreterState = useStoreSubscribe(interpreterStore.state);
    const tape = interpreterState.tape;
    const pointer = interpreterState.pointer;

    // Determine cell size and display parameters
    const cellInfo = tape instanceof Uint8Array
        ? { bits: 8, bytes: 1, max: 255, width: 100 }
        : tape instanceof Uint16Array
            ? { bits: 16, bytes: 2, max: 65535, width: 140 }
            : tape instanceof Uint32Array
                ? { bits: 32, bytes: 4, max: 4294967295, width: 180 }
                : { bits: 8, bytes: 1, max: 255, width: 100 };

    const containerRef = useRef<HTMLDivElement>(null);

    const CELL_WIDTH = cellInfo.width;
    const CELL_HEIGHT = 120;
    const GAP = 8; // Increased gap for better spacing

    const virtualizer = useVirtualizer({
        horizontal: true,
        count: Math.min(tape.length, 10000), // Limit for performance
        getScrollElement: () => containerRef.current,
        estimateSize: () => CELL_WIDTH + GAP,
        overscan: 10,
        paddingStart: 24,
        paddingEnd: 24,
        getItemKey: (index) => index, // Add stable keys
    });

    // Auto-scroll to pointer when it changes
    useEffect(() => {
        if (pointer < 10000) { // Only if within visible range
            virtualizer.scrollToIndex(pointer, {
                align: 'center',
            });
        }
    }, [pointer, virtualizer]);

    return (
        <div className="flex flex-col h-full bg-zinc-950">
            {/* Header */}
            <div className="flex items-center justify-between px-4 py-2 border-b border-zinc-800 bg-zinc-900">
                <div className="flex items-center gap-4">
                    <h3 className="text-sm font-medium text-zinc-300">Memory Tape</h3>
                    <div className="flex items-center gap-2 text-xs text-zinc-500">
                        <span className="rounded-sm px-2 py-0.5 bg-zinc-800">
                            {cellInfo.bits}-bit cells
                        </span>
                        <span>•</span>
                        <span>Pointer: {pointer}</span>
                        <span>•</span>
                        <span>Value: {tape[pointer]}</span>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    <button
                        onClick={() => virtualizer.scrollToIndex(0)}
                        className="rounded-sm text-xs px-3 py-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors"
                    >
                        Go to Start
                    </button>
                    <button
                        onClick={() => virtualizer.scrollToIndex(pointer, {align: 'center'})}
                        className="rounded-sm text-xs px-3 py-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors"
                    >
                        Go to Pointer
                    </button>
                </div>
            </div>

            {/* Tape visualization */}
            <div
                ref={containerRef}
                className="flex-1 overflow-x-auto overflow-y-hidden relative"
                style={{
                    scrollbarWidth: 'thin',
                    scrollbarColor: '#3f3f46 #18181b'
                }}
            >
                <div
                    style={{
                        width: `${virtualizer.getTotalSize()}px`,
                        height: '100%',
                        position: 'relative',
                    }}
                >
                    {virtualizer.getVirtualItems().map((virtualItem) => {
                        const index = virtualItem.index;
                        const value = tape[index];
                        const isPointer = index === pointer;
                        const isNonZero = value !== 0;

                        return (
                            <div
                                key={virtualItem.key}
                                style={{
                                    position: 'absolute',
                                    top: '50%',
                                    left: `${virtualItem.start}px`,
                                    width: `${CELL_WIDTH}px`,
                                    height: `${CELL_HEIGHT}px`,
                                    transform: `translateY(-50%)`,
                                }}
                                className="p-1"
                            >
                                <div
                                    className={clsx(
                                        "relative h-full rounded border transition-all duration-200",
                                        "flex flex-col items-center justify-between py-2 px-1",
                                        {
                                            'border-yellow-500 bg-yellow-950/50 shadow-lg shadow-yellow-500/20 scale-105 z-10': isPointer,
                                            'border-blue-500/50 bg-blue-950/30': isNonZero && !isPointer,
                                            'border-zinc-700 bg-zinc-900/50': !isNonZero && !isPointer,
                                        }
                                    )}
                                >
                                    {/* Cell index */}
                                    <div className={clsx(
                                        "text-xs font-mono",
                                        isPointer ? 'text-yellow-400' : 'text-zinc-600'
                                    )}>
                                        #{index}
                                    </div>

                                    {/* Main value display */}
                                    <div className={clsx(
                                        "text-2xl font-bold font-mono",
                                        {
                                            'text-yellow-300': isPointer,
                                            'text-blue-300': isNonZero && !isPointer,
                                            'text-zinc-500': !isNonZero && !isPointer,
                                        }
                                    )}>
                                        {value}
                                    </div>

                                    {/* Additional representations */}
                                    <div className="space-y-1 text-center w-full">
                                        {/* Hex representation for larger cells */}
                                        {cellInfo.bits > 8 && (
                                            <div className={clsx(
                                                "text-xs font-mono",
                                                isPointer ? 'text-yellow-400/70' : 'text-zinc-600'
                                            )}>
                                                {formatHex(value, cellInfo.bytes)}
                                            </div>
                                        )}

                                        {/* Binary representation */}
                                        <div className={clsx(
                                            "font-mono leading-tight",
                                            cellInfo.bits > 16 ? "text-[9px]" : "text-[10px]",
                                            isPointer ? 'text-yellow-400/70' :
                                                isNonZero ? 'text-blue-400/70' : 'text-zinc-600'
                                        )}>
                                            {formatBinary(value, cellInfo.bits)}
                                        </div>

                                        {/* ASCII for 8-bit printable values */}
                                        {cellInfo.bits === 8 && value >= 32 && value <= 126 && (
                                            <div className={clsx(
                                                "text-xs font-mono",
                                                isPointer ? 'text-yellow-400' :
                                                    'text-zinc-500'
                                            )}>
                                                '{String.fromCharCode(value)}'
                                            </div>
                                        )}
                                    </div>

                                    {/* Pointer indicator */}
                                    {isPointer && (
                                        <div className="absolute -bottom-3 left-1/2 transform -translate-x-1/2">
                                            <div className="w-0 h-0 border-l-[6px] border-r-[6px] border-t-[6px]
                                                          border-transparent border-t-yellow-500"></div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>

            {/* Status bar */}
            <div className="flex items-center justify-between px-4 py-2 border-t border-zinc-800 bg-zinc-900 text-xs text-zinc-500">
                <div className="flex items-center gap-3">
                    <span>Memory: {tape.length.toLocaleString()} cells</span>
                    <span>•</span>
                    <span>Range: 0-{cellInfo.max.toLocaleString()}</span>
                </div>
                <span>Scroll horizontally or use mouse wheel</span>
            </div>
        </div>
    );
}

export function Debugger() {
    return (
        <div className="flex flex-col h-full">
            <Tape/>
        </div>
    )
}