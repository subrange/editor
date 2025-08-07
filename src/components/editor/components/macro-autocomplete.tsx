import { useEffect, useState, useRef } from "react";
import { useStoreSubscribeToField } from "../../../hooks/use-store-subscribe.tsx";
import { EditorStore, type Position } from "../stores/editor.store.ts";
import { type MacroDefinition } from "../../../services/macro-expander/macro-expander.ts";
import { CHAR_HEIGHT, LINE_PADDING_LEFT, LINE_PADDING_TOP } from "../constants.ts";
import clsx from "clsx";

interface MacroAutocompleteProps {
    store: EditorStore;
    macros: MacroDefinition[];
    charWidth: number;
}

function measureCharacterWidth() {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
        throw new Error("Failed to get canvas context");
    }
    context.font = "14px monospace";
    return context.measureText("M").width;
}

export function MacroAutocomplete({ store, macros, charWidth }: MacroAutocompleteProps) {
    const selection = useStoreSubscribeToField(store.editorState, "selection");
    const lines = useStoreSubscribeToField(store.editorState, "lines");
    const [isVisible, setIsVisible] = useState(false);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [filter, setFilter] = useState("");
    const [triggerPosition, setTriggerPosition] = useState<Position | null>(null);
    const [showAbove, setShowAbove] = useState(false);
    const [prefix, setPrefix] = useState<'@' | '#'>('@');
    const menuRef = useRef<HTMLDivElement>(null);
    const scrollContainerRef = useRef<HTMLDivElement>(null);
    const previousLineRef = useRef<string>("");
    const previousCursorRef = useRef<Position>({ line: 0, column: 0 });
    
    // Check if we should show autocomplete
    useEffect(() => {
        const cursorPos = selection.focus;
        if (cursorPos.line >= lines.length) {
            setIsVisible(false);
            return;
        }
        
        const line = lines[cursorPos.line].text;
        const previousLine = previousLineRef.current;
        const previousCursor = previousCursorRef.current;
        
        // Store current state for next comparison
        previousLineRef.current = line;
        previousCursorRef.current = cursorPos;
        
        // Check if we just typed something (line changed or cursor moved forward by 1)
        const justTyped = (
            cursorPos.line === previousCursor.line && 
            cursorPos.column === previousCursor.column + 1 &&
            line !== previousLine
        );
        
        // Only show autocomplete if we just typed
        if (!justTyped && !isVisible) {
            return;
        }
        
        const textBeforeCursor = line.substring(0, cursorPos.column);
        
        // Check if we have @ or # followed by optional word characters
        const match = textBeforeCursor.match(/([@#])(\w*)$/);
        if (match) {
            // Only show if we just typed or if already visible (continuing to type)
            if (justTyped || isVisible) {
                setPrefix(match[1] as '@' | '#');
                setFilter(match[2] || "");
                setTriggerPosition({
                    line: cursorPos.line,
                    column: cursorPos.column - match[0].length
                });
                setIsVisible(true);
                setSelectedIndex(0);
            }
        } else {
            setIsVisible(false);
        }
    }, [selection, lines, isVisible]);
    
    // Filter macros based on input
    const filteredMacros = macros.filter(macro => 
        macro.name.toLowerCase().startsWith(filter.toLowerCase())
    );
    
    // Scroll to selected item when it changes
    useEffect(() => {
        if (!scrollContainerRef.current || !isVisible) return;
        
        const container = scrollContainerRef.current;
        const items = container.children;
        if (items.length === 0 || selectedIndex >= items.length) return;
        
        const selectedItem = items[selectedIndex] as HTMLElement;
        
        // Calculate relative positions within the scroll container
        const itemTop = selectedItem.offsetTop;
        const itemBottom = itemTop + selectedItem.offsetHeight;
        const containerScrollTop = container.scrollTop;
        const containerHeight = container.clientHeight;
        
        // Check if item is out of view and adjust scroll
        if (itemTop < containerScrollTop) {
            // Item is above the visible area - scroll up
            container.scrollTop = itemTop;
        } else if (itemBottom > containerScrollTop + containerHeight) {
            // Item is below the visible area - scroll down
            container.scrollTop = itemBottom - containerHeight;
        }
    }, [selectedIndex, isVisible]);
    
    // Handle keyboard navigation with capture phase to intercept before keybindingsService
    useEffect(() => {
        if (!isVisible) return;
        
        const handleKeyDown = (e: KeyboardEvent) => {
            switch (e.key) {
                case "ArrowDown":
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                    setSelectedIndex(prev => 
                        prev < filteredMacros.length - 1 ? prev + 1 : 0
                    );
                    break;
                    
                case "ArrowUp":
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                    setSelectedIndex(prev => 
                        prev > 0 ? prev - 1 : filteredMacros.length - 1
                    );
                    break;
                    
                case "Enter":
                case "Tab":
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                    if (filteredMacros.length > 0) {
                        insertMacro(filteredMacros[selectedIndex]);
                    }
                    break;
                    
                case "Escape":
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                    setIsVisible(false);
                    break;
            }
        };
        
        // Use capture phase to intercept events before they bubble down
        document.addEventListener("keydown", handleKeyDown, true);
        return () => document.removeEventListener("keydown", handleKeyDown, true);
    }, [isVisible, selectedIndex, filteredMacros]);
    
    // Calculate popup position and height
    const x = triggerPosition ? triggerPosition.column * charWidth + LINE_PADDING_LEFT : 0;
    const lineY = triggerPosition ? triggerPosition.line * CHAR_HEIGHT + LINE_PADDING_TOP : 0;
    // Compact height calculation - each item is ~28px (single line with padding)
    const itemHeight = 28;
    const footerHeight = 26; // Footer with shortcuts
    const popupHeight = Math.min(280, filteredMacros.length * itemHeight + footerHeight);
    
    // Check if we should show above or below
    useEffect(() => {
        if (!menuRef.current || !isVisible || !triggerPosition) return;
        
        const container = menuRef.current.parentElement;
        if (!container) return;
        
        const containerRect = container.getBoundingClientRect();
        const containerScrollTop = container.scrollTop || 0;
        
        // Calculate actual position relative to viewport
        const actualLineY = lineY - containerScrollTop;
        const spaceBelow = containerRect.height - actualLineY - CHAR_HEIGHT;
        const spaceAbove = actualLineY;
        
        // Show above if not enough space below and more space above
        setShowAbove(spaceBelow < popupHeight && spaceAbove > spaceBelow);
    }, [lineY, popupHeight, isVisible, triggerPosition]);

    const insertMacro = (macro: MacroDefinition) => {
        if (!triggerPosition) return;
        
        const cursorPos = selection.focus;
        const line = lines[cursorPos.line].text;
        
        // Build the insertion text with the stored prefix
        let insertText = prefix + macro.name;
        if (macro.parameters && macro.parameters.length > 0) {
            insertText += `(${macro.parameters.join(", ")})`;
        }
        
        // Replace from trigger position to current cursor position
        store.replaceRange(triggerPosition, cursorPos, insertText);
        
        // Move cursor after the inserted text
        const newColumn = triggerPosition.column + insertText.length;
        store.setCursorPosition({ line: cursorPos.line, column: newColumn });
        
        setIsVisible(false);
    };
    
    if (!isVisible || filteredMacros.length === 0 || !triggerPosition) {
        return null;
    }
    
    // Position above or below the line
    const y = showAbove 
        ? lineY - popupHeight - 4  // 4px gap above
        : lineY + CHAR_HEIGHT + 4; // 4px gap below
    
    return (
        <div
            ref={menuRef}
            className="absolute z-50 bg-zinc-900 border border-zinc-700 rounded-lg shadow-2xl overflow-hidden"
            style={{
                left: `${x}px`,
                top: `${y}px`,
                minWidth: '400px',
                maxWidth: '600px',
                maxHeight: `${popupHeight}px`
            }}
            onMouseDown={(e) => {
                // Prevent editor from receiving mouse events
                e.stopPropagation();
            }}
        >
            <div ref={scrollContainerRef} className="overflow-y-auto max-h-48">
                {filteredMacros.map((macro, index) => (
                    <div
                        key={macro.name}
                        className={clsx(
                            "px-2 py-1 cursor-pointer text-sm",
                            index === selectedIndex
                                ? "bg-zinc-800 text-zinc-100"
                                : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
                        )}
                        onMouseEnter={() => setSelectedIndex(index)}
                        onMouseDown={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                        }}
                        onClick={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            insertMacro(macro);
                        }}
                    >
                        <div className="flex items-center gap-2 w-full">
                            <span className={prefix === '@' ? "text-blue-400" : "text-green-400"}>
                                {prefix}
                            </span>
                            <span className="font-mono flex-shrink-0">
                                {macro.name}
                                {macro.parameters && macro.parameters.length > 0 && (
                                    <span className="text-zinc-500 text-xs ml-1">
                                        ({macro.parameters.join(", ")})
                                    </span>
                                )}
                            </span>
                            <span className="text-zinc-600 mx-2">→</span>
                            <span className="text-xs text-zinc-500 truncate font-mono flex-1">
                                {macro.body}
                            </span>
                        </div>
                    </div>
                ))}
            </div>
            {/* Footer with shortcuts */}
            {filteredMacros.length > 0 && (
                <div className="px-2 py-1 border-t border-zinc-700 text-xs text-zinc-500 flex items-center gap-2 text-[10px]">
                    <span>↑↓ Navigate</span>
                    <span>Tab/Enter Select</span>
                    <span>Esc Cancel</span>
                </div>
            )}
        </div>
    );
}