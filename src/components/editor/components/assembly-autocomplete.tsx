import { useState, useEffect, useRef } from 'react';
import clsx from 'clsx';
import { CHAR_HEIGHT } from '../constants.ts';
import { CHAR_WIDTH } from '../../helpers.ts';

interface AssemblyAutocompleteProps {
    visible: boolean;
    x: number;
    y: number;
    currentWord: string;
    labels?: string[];
    onSelect: (completion: string) => void;
    onDismiss: () => void;
}

interface CompletionItem {
    label: string;
    detail: string;
    insertText: string;
    type: 'instruction' | 'register' | 'directive' | 'label';
}

// Assembly instructions with descriptions
const INSTRUCTIONS: CompletionItem[] = [
    { label: 'NOP', detail: 'No operation', insertText: 'NOP', type: 'instruction' },
    { label: 'ADD', detail: 'Add rd = rs + rt', insertText: 'ADD ', type: 'instruction' },
    { label: 'SUB', detail: 'Subtract rd = rs - rt', insertText: 'SUB ', type: 'instruction' },
    { label: 'AND', detail: 'Bitwise AND rd = rs & rt', insertText: 'AND ', type: 'instruction' },
    { label: 'OR', detail: 'Bitwise OR rd = rs | rt', insertText: 'OR ', type: 'instruction' },
    { label: 'XOR', detail: 'Bitwise XOR rd = rs ^ rt', insertText: 'XOR ', type: 'instruction' },
    { label: 'SLL', detail: 'Shift left logical rd = rs << rt', insertText: 'SLL ', type: 'instruction' },
    { label: 'SRL', detail: 'Shift right logical rd = rs >> rt', insertText: 'SRL ', type: 'instruction' },
    { label: 'SLT', detail: 'Set less than rd = (rs < rt)', insertText: 'SLT ', type: 'instruction' },
    { label: 'SLTU', detail: 'Set less than unsigned', insertText: 'SLTU ', type: 'instruction' },
    { label: 'ADDI', detail: 'Add immediate rd = rs + imm', insertText: 'ADDI ', type: 'instruction' },
    { label: 'ANDI', detail: 'AND immediate rd = rs & imm', insertText: 'ANDI ', type: 'instruction' },
    { label: 'ORI', detail: 'OR immediate rd = rs | imm', insertText: 'ORI ', type: 'instruction' },
    { label: 'XORI', detail: 'XOR immediate rd = rs ^ imm', insertText: 'XORI ', type: 'instruction' },
    { label: 'LI', detail: 'Load immediate rd = imm', insertText: 'LI ', type: 'instruction' },
    { label: 'SLLI', detail: 'Shift left logical immediate', insertText: 'SLLI ', type: 'instruction' },
    { label: 'SRLI', detail: 'Shift right logical immediate', insertText: 'SRLI ', type: 'instruction' },
    { label: 'LOAD', detail: 'Load from memory rd = mem[rs + offset]', insertText: 'LOAD ', type: 'instruction' },
    { label: 'STORE', detail: 'Store to memory mem[rs + offset] = rd', insertText: 'STORE ', type: 'instruction' },
    { label: 'JAL', detail: 'Jump and link', insertText: 'JAL ', type: 'instruction' },
    { label: 'JALR', detail: 'Jump and link register', insertText: 'JALR ', type: 'instruction' },
    { label: 'BEQ', detail: 'Branch if equal', insertText: 'BEQ ', type: 'instruction' },
    { label: 'BNE', detail: 'Branch if not equal', insertText: 'BNE ', type: 'instruction' },
    { label: 'BLT', detail: 'Branch if less than', insertText: 'BLT ', type: 'instruction' },
    { label: 'BGE', detail: 'Branch if greater or equal', insertText: 'BGE ', type: 'instruction' },
    { label: 'HALT', detail: 'Stop execution', insertText: 'HALT', type: 'instruction' },
];

// Registers
const REGISTERS: CompletionItem[] = [
    { label: 'R0', detail: 'Zero register (always 0)', insertText: 'R0', type: 'register' },
    { label: 'PC', detail: 'Program counter', insertText: 'PC', type: 'register' },
    { label: 'PCB', detail: 'Program counter bank', insertText: 'PCB', type: 'register' },
    { label: 'RA', detail: 'Return address', insertText: 'RA', type: 'register' },
    { label: 'RAB', detail: 'Return address bank', insertText: 'RAB', type: 'register' },
    { label: 'R3', detail: 'General purpose register 3', insertText: 'R3', type: 'register' },
    { label: 'R4', detail: 'General purpose register 4', insertText: 'R4', type: 'register' },
    { label: 'R5', detail: 'General purpose register 5', insertText: 'R5', type: 'register' },
    { label: 'R6', detail: 'General purpose register 6', insertText: 'R6', type: 'register' },
    { label: 'R7', detail: 'General purpose register 7', insertText: 'R7', type: 'register' },
    { label: 'R8', detail: 'General purpose register 8', insertText: 'R8', type: 'register' },
    { label: 'R9', detail: 'General purpose register 9', insertText: 'R9', type: 'register' },
    { label: 'R10', detail: 'General purpose register 10', insertText: 'R10', type: 'register' },
    { label: 'R11', detail: 'General purpose register 11', insertText: 'R11', type: 'register' },
    { label: 'R12', detail: 'General purpose register 12', insertText: 'R12', type: 'register' },
    { label: 'R13', detail: 'General purpose register 13', insertText: 'R13', type: 'register' },
    { label: 'R14', detail: 'General purpose register 14', insertText: 'R14', type: 'register' },
    { label: 'R15', detail: 'General purpose register 15', insertText: 'R15', type: 'register' },
];

// Directives
const DIRECTIVES: CompletionItem[] = [
    { label: '.data', detail: 'Start data section', insertText: '.data\n', type: 'directive' },
    { label: '.code', detail: 'Start code section', insertText: '.code\n', type: 'directive' },
    { label: '.space', detail: 'Reserve bytes', insertText: '.space ', type: 'directive' },
    { label: '.byte', detail: 'Define byte values', insertText: '.byte ', type: 'directive' },
    { label: '.word', detail: 'Define word values', insertText: '.word ', type: 'directive' },
    { label: '.asciiz', detail: 'Define null-terminated string', insertText: '.asciiz "', type: 'directive' },
    { label: '.ascii', detail: 'Define string', insertText: '.ascii "', type: 'directive' },
];

export function AssemblyAutocomplete({
    visible,
    x,
    y,
    currentWord,
    labels = [],
    onSelect,
    onDismiss
}: AssemblyAutocompleteProps) {
    const [selectedIndex, setSelectedIndex] = useState(0);
    const containerRef = useRef<HTMLDivElement>(null);
    const selectedItemRef = useRef<HTMLDivElement>(null);

    // Create label completions
    const labelCompletions: CompletionItem[] = labels.map(label => ({
        label,
        detail: 'Label',
        insertText: label,
        type: 'label'
    }));

    // Filter completions based on current word
    const completions = [...INSTRUCTIONS, ...REGISTERS, ...DIRECTIVES, ...labelCompletions]
        .filter(item => item.label.toLowerCase().startsWith(currentWord.toLowerCase()))
        .slice(0, 10); // Limit to 10 items

    // Reset selection when completions change
    useEffect(() => {
        setSelectedIndex(0);
    }, [currentWord]);

    // Scroll selected item into view
    useEffect(() => {
        if (selectedItemRef.current && containerRef.current) {
            const container = containerRef.current;
            const item = selectedItemRef.current;
            const itemTop = item.offsetTop;
            const itemBottom = itemTop + item.offsetHeight;
            const containerTop = container.scrollTop;
            const containerBottom = containerTop + container.clientHeight;

            if (itemTop < containerTop) {
                container.scrollTop = itemTop;
            } else if (itemBottom > containerBottom) {
                container.scrollTop = itemBottom - container.clientHeight;
            }
        }
    }, [selectedIndex]);

    // Handle keyboard navigation
    useEffect(() => {
        if (!visible) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            switch (e.key) {
                case 'ArrowDown':
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                    setSelectedIndex(prev => 
                        prev < completions.length - 1 ? prev + 1 : 0
                    );
                    break;
                case 'ArrowUp':
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                    setSelectedIndex(prev => 
                        prev > 0 ? prev - 1 : completions.length - 1
                    );
                    break;
                case 'Enter':
                case 'Tab':
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                    if (completions[selectedIndex]) {
                        onSelect(completions[selectedIndex].insertText);
                    }
                    break;
                case 'Escape':
                    e.preventDefault();
                    e.stopPropagation();
                    e.stopImmediatePropagation();
                    onDismiss();
                    break;
            }
        };

        // Use capture phase to intercept events before they bubble down
        document.addEventListener('keydown', handleKeyDown, true);
        return () => document.removeEventListener('keydown', handleKeyDown, true);
    }, [visible, selectedIndex, completions, onSelect, onDismiss]);

    if (!visible || completions.length === 0) {
        return null;
    }

    const typeColors = {
        instruction: 'text-blue-400',
        register: 'text-green-400',
        directive: 'text-purple-400',
        label: 'text-rose-400'
    };

    return (
        <div
            ref={containerRef}
            className="absolute z-50 bg-zinc-900 border border-zinc-700 rounded-lg shadow-2xl overflow-hidden"
            style={{
                left: `${x}px`,
                top: `${y + CHAR_HEIGHT + 4}px`,
                minWidth: '400px',
                maxWidth: '600px',
                maxHeight: '280px'
            }}
            onMouseDown={(e) => {
                // Prevent editor from receiving mouse events
                e.stopPropagation();
            }}
        >
            <div className="overflow-y-auto max-h-48">
                {completions.map((item, index) => (
                    <div
                        key={item.label}
                        ref={index === selectedIndex ? selectedItemRef : null}
                        className={clsx(
                            "px-2 py-1 cursor-pointer text-sm",
                            index === selectedIndex
                                ? "bg-zinc-800 text-zinc-100"
                                : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
                        )}
                        onClick={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            onSelect(item.insertText);
                        }}
                        onMouseEnter={() => setSelectedIndex(index)}
                        onMouseDown={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                        }}
                    >
                        <div className="flex items-center gap-2 w-full">
                            <span className={clsx("font-mono flex-shrink-0", typeColors[item.type])}>
                                {item.label}
                            </span>
                            <span className="text-zinc-600 mx-2">→</span>
                            <span className="text-xs text-zinc-500 truncate font-mono flex-1">
                                {item.detail}
                            </span>
                        </div>
                    </div>
                ))}
            </div>
            {/* Footer with shortcuts */}
            {completions.length > 0 && (
                <div className="px-2 py-1 border-t border-zinc-700 text-xs text-zinc-500 flex items-center gap-2 text-[10px]">
                    <span>↑↓ Navigate</span>
                    <span>Tab/Enter Select</span>
                    <span>Esc Cancel</span>
                </div>
            )}
        </div>
    );
}