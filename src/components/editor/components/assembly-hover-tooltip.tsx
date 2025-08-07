import { useState, useEffect, useRef } from 'react';
import { type AssemblyToken } from '../services/assembly-tokenizer.ts';

interface AssemblyHoverTooltipProps {
    token: AssemblyToken;
    x: number;
    y: number;
    visible: boolean;
}

// Instruction descriptions
const instructionHelp: Record<string, { syntax: string; description: string }> = {
    // Arithmetic
    'NOP': { syntax: 'NOP', description: 'No operation - does nothing' },
    'ADD': { syntax: 'ADD rd, rs, rt', description: 'Add: rd = rs + rt' },
    'SUB': { syntax: 'SUB rd, rs, rt', description: 'Subtract: rd = rs - rt' },
    'AND': { syntax: 'AND rd, rs, rt', description: 'Bitwise AND: rd = rs & rt' },
    'OR': { syntax: 'OR rd, rs, rt', description: 'Bitwise OR: rd = rs | rt' },
    'XOR': { syntax: 'XOR rd, rs, rt', description: 'Bitwise XOR: rd = rs ^ rt' },
    'SLL': { syntax: 'SLL rd, rs, rt', description: 'Shift left logical: rd = rs << rt' },
    'SRL': { syntax: 'SRL rd, rs, rt', description: 'Shift right logical: rd = rs >> rt' },
    'SLT': { syntax: 'SLT rd, rs, rt', description: 'Set less than: rd = (rs < rt) ? 1 : 0' },
    'SLTU': { syntax: 'SLTU rd, rs, rt', description: 'Set less than unsigned: rd = (rs < rt) ? 1 : 0' },
    
    // Immediate
    'ADDI': { syntax: 'ADDI rd, rs, imm', description: 'Add immediate: rd = rs + imm' },
    'ANDI': { syntax: 'ANDI rd, rs, imm', description: 'AND immediate: rd = rs & imm' },
    'ORI': { syntax: 'ORI rd, rs, imm', description: 'OR immediate: rd = rs | imm' },
    'XORI': { syntax: 'XORI rd, rs, imm', description: 'XOR immediate: rd = rs ^ imm' },
    'LI': { syntax: 'LI rd, imm', description: 'Load immediate: rd = imm' },
    'SLLI': { syntax: 'SLLI rd, rs, imm', description: 'Shift left logical immediate: rd = rs << imm' },
    'SRLI': { syntax: 'SRLI rd, rs, imm', description: 'Shift right logical immediate: rd = rs >> imm' },
    
    // Memory
    'LOAD': { syntax: 'LOAD rd, offset, label', description: 'Load from memory: rd = memory[offset + label_address]' },
    'STORE': { syntax: 'STORE rs, offset, label', description: 'Store to memory: memory[offset + label_address] = rs' },
    
    // Control flow
    'JAL': { syntax: 'JAL rd, label', description: 'Jump and link: rd = PC+1; PC = label' },
    'JALR': { syntax: 'JALR rd, rs', description: 'Jump and link register: rd = PC+1; PC = rs' },
    'BEQ': { syntax: 'BEQ rs, rt, label', description: 'Branch if equal: if (rs == rt) PC = label' },
    'BNE': { syntax: 'BNE rs, rt, label', description: 'Branch if not equal: if (rs != rt) PC = label' },
    'BLT': { syntax: 'BLT rs, rt, label', description: 'Branch if less than: if (rs < rt) PC = label' },
    'BGE': { syntax: 'BGE rs, rt, label', description: 'Branch if greater or equal: if (rs >= rt) PC = label' },
    'HALT': { syntax: 'HALT', description: 'Stop execution' },
};

const registerHelp: Record<string, string> = {
    'R0': 'Zero register - always contains 0',
    'PC': 'Program Counter - current instruction address',
    'PCB': 'Program Counter Bank - current bank number',
    'RA': 'Return Address - used by JAL/JALR',
    'RAB': 'Return Address Bank',
    'R3': 'General purpose register 3',
    'R4': 'General purpose register 4',
    'R5': 'General purpose register 5',
    'R6': 'General purpose register 6',
    'R7': 'General purpose register 7',
    'R8': 'General purpose register 8',
    'R9': 'General purpose register 9',
    'R10': 'General purpose register 10',
    'R11': 'General purpose register 11',
    'R12': 'General purpose register 12',
    'R13': 'General purpose register 13',
    'R14': 'General purpose register 14',
    'R15': 'General purpose register 15',
};

const directiveHelp: Record<string, { syntax: string; description: string }> = {
    '.data': { syntax: '.data', description: 'Start data section - define constants and variables' },
    '.code': { syntax: '.code', description: 'Start code section - define instructions' },
    '.space': { syntax: '.space count', description: 'Reserve count bytes of space' },
    '.byte': { syntax: '.byte value1, value2, ...', description: 'Define byte values (8-bit)' },
    '.word': { syntax: '.word value1, value2, ...', description: 'Define word values (16-bit)' },
    '.asciiz': { syntax: '.asciiz "string"', description: 'Define null-terminated string' },
    '.ascii': { syntax: '.ascii "string"', description: 'Define string without null terminator' },
};

export function AssemblyHoverTooltip({ token, x, y, visible }: AssemblyHoverTooltipProps) {
    const [showTooltip, setShowTooltip] = useState(false);
    const tooltipRef = useRef<HTMLDivElement>(null);
    
    useEffect(() => {
        if (visible && (token.type === 'instruction' || token.type === 'register' || token.type === 'directive')) {
            const timer = setTimeout(() => setShowTooltip(true), 300); // Small delay
            return () => clearTimeout(timer);
        } else {
            setShowTooltip(false);
        }
    }, [visible, token]);
    
    if (!showTooltip) return null;
    
    let content: { title: string; syntax?: string; description: string } | null = null;
    
    if (token.type === 'instruction') {
        const help = instructionHelp[token.value.toUpperCase()];
        if (help) {
            content = {
                title: token.value.toUpperCase(),
                syntax: help.syntax,
                description: help.description
            };
        }
    } else if (token.type === 'register') {
        const help = registerHelp[token.value.toUpperCase()];
        if (help) {
            content = {
                title: token.value.toUpperCase(),
                description: help
            };
        }
    } else if (token.type === 'directive') {
        const help = directiveHelp[token.value.toLowerCase()];
        if (help) {
            content = {
                title: token.value,
                syntax: help.syntax,
                description: help.description
            };
        }
    }
    
    if (!content) return null;
    
    return (
        <div
            ref={tooltipRef}
            className="absolute z-50 pointer-events-none"
            style={{
                left: `${x}px`,
                top: `${y - 10}px`, // Position above the token
                transform: 'translateY(-100%)'
            }}
        >
            <div className="bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl p-3 max-w-md">
                <div className="text-sm">
                    <div className="font-bold text-zinc-200 mb-1">{content.title}</div>
                    {content.syntax && (
                        <div className="font-mono text-xs text-blue-400 mb-2">{content.syntax}</div>
                    )}
                    <div className="text-zinc-400 text-xs">{content.description}</div>
                </div>
            </div>
        </div>
    );
}