import {interpreterStore} from "./interpreter.store.ts";
import {useStoreSubscribe} from "../../hooks/use-store-subscribe.tsx";
import {useVirtualizer} from '@tanstack/react-virtual';
import {useRef, useEffect} from 'react';
import clsx from "clsx";
import {settingsStore} from "../../stores/settings.store.ts";

// Lane color palette - 10 distinct colors that work with dark theme
const LANE_COLORS = [
    // Lane 0: Emerald
    { border: 'border-emerald-500/50', bg: 'bg-emerald-950/30' },
    // Lane 1: Sky
    { border: 'border-sky-500/50', bg: 'bg-sky-950/30' },
    // Lane 2: Violet
    { border: 'border-violet-500/50', bg: 'bg-violet-950/30' },
    // Lane 3: Rose
    { border: 'border-rose-500/50', bg: 'bg-rose-950/30' },
    // Lane 4: Amber
    { border: 'border-amber-500/50', bg: 'bg-amber-950/30' },
    // Lane 5: Cyan
    { border: 'border-cyan-500/50', bg: 'bg-cyan-950/30' },
    // Lane 6: Fuchsia
    { border: 'border-fuchsia-500/50', bg: 'bg-fuchsia-950/30' },
    // Lane 7: Lime
    { border: 'border-lime-500/50', bg: 'bg-lime-950/30' },
    // Lane 8: Orange
    { border: 'border-orange-500/50', bg: 'bg-orange-950/30' },
    // Lane 9: Indigo
    { border: 'border-indigo-500/50', bg: 'bg-indigo-950/30' },
];

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
    const settings = useStoreSubscribe(settingsStore.settings);
    const tape = interpreterState.tape;
    const pointer = interpreterState.pointer;
    const laneCount = interpreterState.laneCount;
    const compactView = settings?.debugger.compactView ?? false;

    // Determine cell size and display parameters
    const cellInfo = tape instanceof Uint8Array
        ? { bits: 8, bytes: 1, max: 255, width: 100, compactWidth: 48 }
        : tape instanceof Uint16Array
            ? { bits: 16, bytes: 2, max: 65535, width: 140, compactWidth: 64 }
            : tape instanceof Uint32Array
                ? { bits: 32, bytes: 4, max: 4294967295, width: 180, compactWidth: 84 }
                : { bits: 8, bytes: 1, max: 255, width: 100, compactWidth: 48 };

    const containerRef = useRef<HTMLDivElement>(null);

    const CELL_WIDTH = compactView ? cellInfo.compactWidth : cellInfo.width;
    const CELL_HEIGHT = compactView ? 40 : 120;
    const GAP = compactView ? 2 : 8; // Increased gap for better spacing

    const virtualizer = useVirtualizer({
        horizontal: true,
        count: tape.length,
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
            <div className={clsx(
                "flex items-center justify-between border-b border-zinc-800 bg-zinc-900",
                compactView ? "px-2 py-1" : "px-4 py-2"
            )}>
                <div className={clsx("flex items-center", compactView ? "gap-2" : "gap-4")}>
                    <h3 className={clsx("font-medium text-zinc-300", compactView ? "text-xs" : "text-sm")}>Memory Tape</h3>
                    <div className={clsx("flex items-center gap-2 text-zinc-500", compactView ? "text-[10px]" : "text-xs")}>
                        <span className={clsx("rounded-sm bg-zinc-800", compactView ? "px-1 py-0" : "px-2 py-0.5")}>
                            {cellInfo.bits}-bit cells
                        </span>
                        <span>•</span>
                        <span>Pointer: {pointer}</span>
                        <span>•</span>
                        <span>Value: {tape[pointer]}</span>
                    </div>
                </div>
                <div className={clsx("flex items-center", compactView ? "gap-1" : "gap-2")}>
                    <button
                        onClick={() => virtualizer.scrollToIndex(0)}
                        className={clsx(
                            "rounded-sm bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors",
                            compactView ? "text-[10px] px-2 py-0.5" : "text-xs px-3 py-1"
                        )}
                    >
                        {compactView ? "Start" : "Go to Start"}
                    </button>
                    <button
                        onClick={() => virtualizer.scrollToIndex(pointer, {align: 'center'})}
                        className={clsx(
                            "rounded-sm bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors",
                            compactView ? "text-[10px] px-2 py-0.5" : "text-xs px-3 py-1"
                        )}
                    >
                        {compactView ? "Pointer" : "Go to Pointer"}
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
                        const lane = laneCount > 1 ? index % laneCount : -1;
                        const laneColor = lane >= 0 ? LANE_COLORS[lane] : null;

                        return (
                            <div
                                key={virtualItem.key}
                                style={{
                                    position: 'absolute',
                                    top: compactView ? '50%' : '50%',
                                    left: `${virtualItem.start}px`,
                                    width: `${CELL_WIDTH}px`,
                                    height: `${CELL_HEIGHT}px`,
                                    transform: `translateY(-50%)`,
                                }}
                                className={compactView ? "p-0.5" : "p-1"}
                            >
                                <div
                                    className={clsx(
                                        "relative h-full rounded border transition-all duration-200",
                                        compactView ? "flex flex-col items-center justify-center py-1 px-1" : "flex flex-col items-center justify-between py-2 px-1",
                                        {
                                            // Pointer styles take precedence
                                            'border-yellow-500 bg-yellow-950/50 shadow-lg shadow-yellow-500/20 z-10': isPointer,
                                            'scale-105': isPointer && !compactView,
                                            // Lane colors for multi-lane mode (when not pointer)
                                            [laneColor?.border || '']: laneColor && !isPointer,
                                            [laneColor?.bg || '']: laneColor && !isPointer,
                                            // Default styles for single-lane mode (when not pointer)
                                            'border-blue-500/50 bg-blue-950/30': !laneColor && isNonZero && !isPointer,
                                            'border-zinc-700 bg-zinc-900/50': !laneColor && !isNonZero && !isPointer,
                                        }
                                    )}
                                >
                                    {compactView ? (
                                        <>
                                            {/* Compact view: just index and value */}
                                            <div className={clsx(
                                                "text-[9px] font-mono leading-none",
                                                isPointer ? 'text-yellow-400' : 'text-zinc-600'
                                            )}>
                                                {index}
                                            </div>
                                            <div className={clsx(
                                                "font-bold font-mono",
                                                cellInfo.bits > 16 ? "text-xs" : "text-sm",
                                                {
                                                    'text-yellow-300': isPointer,
                                                    'text-blue-300': isNonZero && !isPointer,
                                                    'text-zinc-500': !isNonZero && !isPointer,
                                                }
                                            )}>
                                                {value}
                                            </div>
                                        </>
                                    ) : (
                                        <>
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
                                        </>
                                    )}

                                    {/* Pointer indicator */}
                                    {isPointer && !compactView && (
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
            <div className={clsx(
                "flex items-center justify-between border-t border-zinc-800 bg-zinc-900 text-zinc-500",
                compactView ? "px-2 py-0.5 text-[10px]" : "px-4 py-2 text-xs"
            )}>
                <div className="flex items-center gap-3">
                    <span>Memory: {tape.length.toLocaleString()} cells</span>
                    <span>•</span>
                    <span>Range: 0-{cellInfo.max.toLocaleString()}</span>
                </div>
                {!compactView && <span>Scroll horizontally or use mouse wheel</span>}
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