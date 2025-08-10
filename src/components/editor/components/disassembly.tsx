import { useEffect, useMemo, useRef, useState } from 'react';
import { useStoreSubscribe } from '../../../hooks/use-store-subscribe';
import { interpreterStore } from '../../debugger/interpreter-facade.store';
import { disassembler } from '../../../services/ripple-assembler';
import { AssemblyTokenizer, assemblyTokenStyles } from '../services/assembly-tokenizer';
import clsx from 'clsx';

interface DisassemblyProps {
  outputRef?: React.RefObject<HTMLDivElement | null>;
  isActive: boolean;
}

export function Disassembly({ outputRef, isActive }: DisassemblyProps) {
  const interpreterState = useStoreSubscribe(interpreterStore.state);
  const { tape, pointer } = interpreterState;
  const [currentInstructionIndex, setCurrentInstructionIndex] = useState<number | null>(null);
  const [watchCells, setWatchCells] = useState<number[]>(() => {
    const saved = localStorage.getItem('disassembly-watch-cells');
    return saved ? JSON.parse(saved) : [];
  });
  const [newWatchCell, setNewWatchCell] = useState('');
  const tokenizer = useMemo(() => new AssemblyTokenizer(), [])
  
  // Constants for instruction layout
  const INSTRUCTION_START = 168;
  const INSTRUCTION_SIZE = 32; // Each instruction spans 32 cells
  
  // Force update when interpreter state changes by using a key that changes
  // This ensures we re-render when the tape is modified
  const updateKey = useMemo(() => {
    // Create a simple hash of key instruction positions
    let hash = 0;
    for (let i = 0; i < 10; i++) {
      const opcodeIndex = INSTRUCTION_START + 27 + (i * 32);
      if (opcodeIndex < tape.length) {
        hash = (hash * 31 + tape[opcodeIndex]) >>> 0;
      }
    }
    return hash;
  }, [interpreterState, tape]);
  
  // Generate disassembled code
  const disassembledCode = useMemo(() => {
    const lines: string[] = [];
    const instructions: Array<{ index: number; text: string; isHalt: boolean }> = [];
    const maxInstructions = 1000; // Limit to prevent excessive processing
    let lastNonHaltIndex = -1;
    
    // First pass: collect all instructions and track last non-HALT
    for (let i = 0; i < maxInstructions; i++) {
      // Calculate component positions for this instruction
      const op3Index = INSTRUCTION_START + 3 + (i * 32);
      const op2Index = op3Index + 8;
      const op1Index = op2Index + 8;
      const opcodeIndex = op1Index + 8;
      
      // Check if we're past the tape bounds
      if (opcodeIndex >= tape.length) break;
      
      // Get instruction components
      const opcode = tape[opcodeIndex];
      const op1 = tape[op1Index];
      const op2 = tape[op2Index];
      const op3 = tape[op3Index];
      
      // Disassemble the instruction
      const disassembledParts = disassembler.disassemble(opcode, op1, op2, op3);
      const [mnemonic, ...operands] = disassembledParts;
      
      // Check if this is a HALT instruction
      const isHalt = mnemonic === 'HALT' || (opcode === 0 && op1 === 0 && op2 === 0 && op3 === 0);
      
      // Format the instruction line with instruction number on the left
      const validOperands = operands.filter(op => op !== null);
      
      // Align mnemonic to 5 characters (longest is 5 letters)
      const alignedMnemonic = mnemonic.padEnd(5, ' ');
      const instructionText = validOperands.length > 0 
        ? `${alignedMnemonic} ${validOperands.join(', ')}`
        : alignedMnemonic;
      
      // Format with instruction number padded to 3 digits
      const formattedLine = `${i.toString().padStart(3, ' ')}:    ${instructionText}`;
      
      instructions.push({ index: i, text: formattedLine, isHalt });
      
      if (!isHalt) {
        lastNonHaltIndex = i;
      }
    }
    
    // Second pass: build final output
    // Include all instructions up to the last non-HALT, plus up to 3 more HALTs
    let haltCount = 0;
    const maxTrailingHalts = 10;
    
    for (const inst of instructions) {
      if (inst.index <= lastNonHaltIndex) {
        // Include all instructions up to the last non-HALT
        lines.push(inst.text);
      } else if (inst.isHalt && haltCount < maxTrailingHalts) {
        // Include a few trailing HALTs
        lines.push(inst.text);
        haltCount++;
      }
    }
    
    // If there were more HALTs, add an ellipsis
    if (instructions.length > lastNonHaltIndex + 1 + maxTrailingHalts) {
      lines.push('        ...');
    }
    
    return lines.join('\n');
  }, [tape, updateKey]);
  
  // Calculate current instruction index based on pointer position
  useEffect(() => {
    // Check if pointer is in the instruction area
    const offset = pointer - INSTRUCTION_START;
    
    if (offset >= 0 && offset < tape.length - INSTRUCTION_START) {
      // Determine which instruction the pointer is in
      const instructionIndex = Math.floor(offset / INSTRUCTION_SIZE);
      setCurrentInstructionIndex(instructionIndex);
    } else {
      setCurrentInstructionIndex(null);
    }
  }, [pointer, tape.length]);
  
  // Tokenize the disassembled code
  const tokenizedLines = useMemo(() => {
    const lines = disassembledCode.split('\n');
    return tokenizer.tokenizeAllLines(lines);
  }, [disassembledCode, tokenizer]);
  
  // Scroll to current instruction when it changes
  useEffect(() => {
    if (isActive && outputRef?.current && currentInstructionIndex !== null) {
      // Find the line element for the current instruction by instruction index
      const lineElement = outputRef.current.querySelector(`[data-instruction-index="${currentInstructionIndex}"]`);
      
      if (lineElement) {
        lineElement.scrollIntoView({ block: 'center' });
      }
    }
  }, [currentInstructionIndex, isActive, outputRef]);
  
  // Extract register values from tape
  const registerValues = useMemo(() => {
    const REGISTER_START = 9; // 8 + 1
    const REGISTER_SPACING = 8;
    const registerNames = ['R0', 'PC', 'PCB', 'RA', 'RAB', 'R3', 'R4', 'R5', 'R6', 'R7', 'R8', 'R9', 'R10', 'R11', 'R12', 'R13', 'R14', 'R15'];
    const registers: Array<{ name: string; value: number; index: number }> = [];
    
    for (let i = 0; i < registerNames.length; i++) {
      const tapeIndex = REGISTER_START + (i * REGISTER_SPACING);
      if (tapeIndex < tape.length) {
        registers.push({
          name: registerNames[i],
          value: tape[tapeIndex],
          index: tapeIndex
        });
      }
    }
    
    return registers;
  }, [tape, interpreterState]); // Depend on interpreterState to catch all updates
  
  // Save watch cells to localStorage whenever they change
  useEffect(() => {
    localStorage.setItem('disassembly-watch-cells', JSON.stringify(watchCells));
  }, [watchCells]);
  
  // Listen for watch cells updates from the tape canvas
  useEffect(() => {
    const handleWatchCellsUpdated = (event: CustomEvent) => {
      const updatedWatchCells = event.detail.watchCells;
      setWatchCells(updatedWatchCells);
    };
    
    window.addEventListener('watchCellsUpdated', handleWatchCellsUpdated as EventListener);
    
    return () => {
      window.removeEventListener('watchCellsUpdated', handleWatchCellsUpdated as EventListener);
    };
  }, []);
  
  // Add a new watch cell
  const addWatchCell = () => {
    const cellNumber = parseInt(newWatchCell, 10);
    if (!isNaN(cellNumber) && cellNumber >= 0 && cellNumber < tape.length && !watchCells.includes(cellNumber)) {
      setWatchCells([...watchCells, cellNumber].sort((a, b) => a - b));
      setNewWatchCell('');
    }
  };
  
  // Remove a watch cell
  const removeWatchCell = (cellNumber: number) => {
    setWatchCells(watchCells.filter(c => c !== cellNumber));
  };
  
  // Function to scroll tape to a specific index
  const scrollToTapeIndex = (index: number) => {
    // Find the canvas element in the debugger
    const canvas = document.querySelector('.tape-canvas-renderer canvas') || 
                   document.querySelector('canvas');
    if (canvas) {
      canvas.dispatchEvent(new CustomEvent('scrollToIndex', { detail: { index } }));
    }
  };
  
  return (
    <div className="text-xs font-mono">
      {/* Register display section - sticky at top */}
      <div className="sticky top-0 bg-zinc-950 border-b border-zinc-800 pb-2 pt-2 -mx-2 px-2 z-10">
        <div className="text-zinc-400 font-bold mb-1">Registers:</div>
        
        {/* R0 */}
        <div className="flex items-baseline gap-1 mb-1">
          <span 
            className={assemblyTokenStyles.register + " cursor-pointer hover:underline"}
            onClick={() => scrollToTapeIndex(registerValues[0]?.index || 9)}
          >
            R0:
          </span>
          <span className={assemblyTokenStyles.number}>{registerValues[0]?.value || 0}</span>
        </div>
        
        {/* PC and PCB */}
        <div className="flex gap-4 mb-1">
          <div className="flex items-baseline gap-1">
            <span 
              className={assemblyTokenStyles.register + " cursor-pointer hover:underline"}
              onClick={() => scrollToTapeIndex(registerValues[1]?.index || 17)}
            >
              PC:
            </span>
            <span className={assemblyTokenStyles.number}>{registerValues[1]?.value || 0}</span>
          </div>
          <div className="flex items-baseline gap-1">
            <span 
              className={assemblyTokenStyles.register + " cursor-pointer hover:underline"}
              onClick={() => scrollToTapeIndex(registerValues[2]?.index || 25)}
            >
              PCB:
            </span>
            <span className={assemblyTokenStyles.number}>{registerValues[2]?.value || 0}</span>
          </div>
        </div>
        
        {/* RA and RAB */}
        <div className="flex gap-4 mb-1">
          <div className="flex items-baseline gap-1">
            <span 
              className={assemblyTokenStyles.register + " cursor-pointer hover:underline"}
              onClick={() => scrollToTapeIndex(registerValues[3]?.index || 33)}
            >
              RA:
            </span>
            <span className={assemblyTokenStyles.number}>{registerValues[3]?.value || 0}</span>
          </div>
          <div className="flex items-baseline gap-1">
            <span 
              className={assemblyTokenStyles.register + " cursor-pointer hover:underline"}
              onClick={() => scrollToTapeIndex(registerValues[4]?.index || 41)}
            >
              RAB:
            </span>
            <span className={assemblyTokenStyles.number}>{registerValues[4]?.value || 0}</span>
          </div>
        </div>
        
        {/* General purpose registers R3-R15 */}
        <div className="flex flex-wrap gap-x-4 gap-y-1">
          {registerValues.slice(5).map(reg => (
            <div key={reg.name} className="flex items-baseline gap-1">
              <span 
                className={assemblyTokenStyles.register + " cursor-pointer hover:underline"}
                onClick={() => scrollToTapeIndex(reg.index)}
              >
                {reg.name}:
              </span>
              <span className={assemblyTokenStyles.number}>{reg.value}</span>
            </div>
          ))}
        </div>
        
        {/* Watch cells section */}
        <div className="mt-2 pt-2 border-t border-zinc-800">
          <div className="text-zinc-400 font-bold mb-1">Watch Cells:</div>
          
          {/* Add new watch cell input */}
          <div className="flex gap-2 mb-2">
            <input
              type="number"
              value={newWatchCell}
              onChange={(e) => setNewWatchCell(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  addWatchCell();
                }
              }}
              placeholder="Cell #"
              className="bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-xs w-20 text-zinc-300"
            />
            <button
              onClick={addWatchCell}
              className="bg-zinc-800 hover:bg-zinc-700 rounded px-2 py-1 text-xs text-zinc-300"
            >
              Add
            </button>
          </div>
          
          {/* Display watched cells */}
          {watchCells.length > 0 ? (
            <div className="flex flex-wrap gap-x-4 gap-y-1">
              {watchCells.map(cellIndex => (
                <div key={cellIndex} className="flex items-baseline gap-1">
                  <span 
                    className="text-blue-400 cursor-pointer hover:underline"
                    onClick={() => scrollToTapeIndex(cellIndex)}
                  >
                    [{cellIndex}]:
                  </span>
                  <span className={assemblyTokenStyles.number}>
                    {cellIndex < tape.length ? tape[cellIndex] : 0}
                  </span>
                  <button
                    onClick={() => removeWatchCell(cellIndex)}
                    className="text-zinc-500 hover:text-red-400 ml-1"
                    title="Remove watch"
                  >
                    ×
                  </button>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-zinc-500 italic">No cells being watched</div>
          )}
        </div>
      </div>
      
      {/* Instructions section */}
      <div>
        <div className="text-zinc-400 font-bold mb-1">Instructions:</div>
        {tokenizedLines.map((tokens, lineIndex) => {
        const lineText = disassembledCode.split('\n')[lineIndex];
        
        // Check if this is the ellipsis line
        const isEllipsis = lineText?.trim() === '...';
        
        // Extract instruction number from the line if it's not ellipsis
        let instructionNumber: number | null = null;
        if (!isEllipsis && lineText) {
          const match = lineText.match(/^\s*(\d+):/);
          if (match) {
            instructionNumber = parseInt(match[1], 10);
          }
        }
        
        // Check if this line corresponds to the current instruction
        const isCurrentInstruction = currentInstructionIndex !== null && 
                                   instructionNumber === currentInstructionIndex;
        
        // Calculate the tape index for this instruction
        const instructionTapeIndex = instructionNumber !== null 
          ? INSTRUCTION_START + 3 + (instructionNumber * 32) // Start of instruction (op3)
          : null;
        
        return (
          <div
            key={lineIndex}
            data-line-index={lineIndex}
            data-instruction-index={instructionNumber}
            className={clsx(
              "px-2 py-0.5",
              isCurrentInstruction && "bg-yellow-500/20 border-l-2 border-yellow-500",
              instructionTapeIndex !== null && "cursor-pointer hover:bg-zinc-800/50"
            )}
            onClick={() => {
              if (instructionTapeIndex !== null) {
                scrollToTapeIndex(instructionTapeIndex);
              }
            }}
          >
            {isEllipsis ? (
              <span className="text-zinc-500 pl-8">...</span>
            ) : tokens.length === 0 ? (
              <span className="text-zinc-600">&nbsp;</span>
            ) : (
              tokens.map((token, tokenIndex) => (
                <span
                  key={tokenIndex}
                  className={assemblyTokenStyles[token.type]}
                >
                  {token.value}
                </span>
              ))
            )}
          </div>
        );
      })}
      </div>
    </div>
  );
}