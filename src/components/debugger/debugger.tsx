import {interpreterStore} from "./interpreter.store.ts";
import {useStoreSubscribe} from "../../hooks/use-store-subscribe.tsx";
import {useVirtualizer} from '@tanstack/react-virtual';
import {useRef, useEffect} from 'react';
import {Toolbar} from "./toolbar.tsx";

type ToolbarButtonProps = {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    onClick: () => void;
    disabled?: boolean;
    variant?: 'default' | 'success' | 'danger' | 'warning' | 'info';
}

function Tape() {
    const interpreterState = useStoreSubscribe(interpreterStore.state);
    const tape = interpreterState.tape;

    const pointer = interpreterState.pointer;

    const containerRef = useRef<HTMLDivElement>(null);

    // Cell dimensions
    const CELL_WIDTH = 100;
    const CELL_HEIGHT = 120;
    const GAP = 4;

    const virtualizer = useVirtualizer({
        horizontal: true,
        count: tape.length,
        getScrollElement: () => containerRef.current,
        estimateSize: () => CELL_WIDTH + GAP,
        overscan: 10, // Render 10 extra items on each side
    });

    // Auto-scroll to pointer when it changes
    useEffect(() => {
        virtualizer.scrollToIndex(pointer, {
            align: 'center',
        });
    }, [pointer, virtualizer]);

    return (
        <div className="v grow-1 bg-zinc-950 border-t border-zinc-800">
            <div
                ref={containerRef}
                className="grow-1 overflow-x-auto overflow-y-hidden"
                style={{
                    // Hide scrollbar but keep functionality
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
                                    left: 0,
                                    width: `${CELL_WIDTH}px`,
                                    height: `${CELL_HEIGHT}px`,
                                    transform: `translateX(${virtualItem.start}px) translateY(-50%)`,
                                }}
                                className="p-1"
                            >
                                <div
                                    className={`
                                        relative h-full rounded-lg border-2 transition-all duration-200
                                        grid grid-cols-1 grid-rows-4
                                        ${isPointer
                                        ? 'border-yellow-500 bg-yellow-950 shadow-lg shadow-yellow-500/20 scale-110 z-10'
                                        : isNonZero
                                            ? 'border-blue-600 bg-blue-950'
                                            : 'border-zinc-700 bg-zinc-900'
                                    }
                                    `}
                                >
                                    {/* Cell index */}
                                    <div className={`
                                        flex items-center justify-center
                                        text-xs font-mono
                                        ${isPointer ? 'text-yellow-300' : 'text-zinc-600'}
                                    `}>
                                        {index}
                                    </div>

                                    {/* Cell value */}
                                    <div className={`
                                        flex items-center justify-center
                                        text-2xl font-bold font-mono text-center
                                        ${isPointer
                                        ? 'text-yellow-200'
                                        : isNonZero
                                            ? 'text-blue-300'
                                            : 'text-zinc-400'
                                    }
                                    `}>
                                        {value}
                                    </div>

                                    {/* Binary representation */}
                                    <div className={`
                                        flex items-center justify-center
                                        text-sm font-mono text-center
                                        ${isPointer ? 'text-yellow-400' : isNonZero
                                        ? 'text-blue-300'
                                        : 'text-zinc-500'}
                                    `}>
                                        {value.toString(2).padStart(8, '0')}
                                    </div>

                                    {/* ASCII representation if printable */}
                                    {value >= 32 && value <= 126 && (
                                        <div className={`
                                            flex items-center justify-center
                                            text-xs font-mono text-center
                                            ${isPointer ? 'text-yellow-400' : isNonZero
                                            ? 'text-blue-300'
                                            : 'text-zinc-500'}
                                        `}>
                                            '{String.fromCharCode(value)}'
                                        </div>
                                    )}

                                    {/* Pointer indicator */}
                                    {isPointer && (
                                        <div className="absolute -bottom-3 left-1/2 transform -translate-x-1/2">
                                            <div className="w-0 h-0 border-l-4 border-r-4 border-t-4
                                                          border-transparent border-t-yellow-500"></div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>

            {/* Quick jump controls */}
            <div className="h-10 bg-zinc-950 border-t border-zinc-800 flex items-center px-2 gap-2">
                <button
                    onClick={() => virtualizer.scrollToIndex(0)}
                    className="text-xs px-2 py-1 bg-zinc-800 hover:bg-zinc-700 rounded text-zinc-400"
                >
                    Go to Start
                </button>
                <button
                    onClick={() => virtualizer.scrollToIndex(pointer, {align: 'center'})}
                    className="text-xs px-2 py-1 bg-zinc-800 hover:bg-zinc-700 rounded text-zinc-400"
                >
                    Go to Pointer
                </button>
                <div className="ml-auto text-xs text-zinc-500">
                    Scroll with mouse wheel or drag
                </div>
            </div>
        </div>
    );
}

export function Debugger() {
    return (
        <div className="v grow-1">
            <Tape/>
            <Toolbar/>
        </div>
    )
}