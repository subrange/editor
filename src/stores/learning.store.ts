import { BehaviorSubject } from 'rxjs';

// Import learning content files using Vite's ?raw suffix
// IDE content
import ideWelcome from '../learning-content/ide/basics/welcome.bf?raw';
import ideEditorsMacro from '../learning-content/ide/basics/editors-macro.bfm?raw';

// Import new documentation files
// Note: These will be loaded dynamically via markdown viewer

// Brainfuck content
import bfHelloWorld from '../learning-content/brainfuck/basics/hello-world.bf?raw';
import bfCommands from '../learning-content/brainfuck/basics/commands.bf?raw';
import bfLoops from '../learning-content/brainfuck/basics/loops.bf?raw';
import bfMandelbrot from '../learning-content/brainfuck/examples/mandelbrot.bf?raw';
import bfSierpinski from '../learning-content/brainfuck/examples/sierpinski.bf?raw';

// Macro content
import macroIntro from '../learning-content/macro/basics/intro.bfm?raw';
import macroRipple from '../learning-content/macro/advanced/ripple-hello-world.bfm?raw';

// RVM content
// import rvmIntro from '../learning-content/rvm/basics/intro.asm?raw';

// Types for the learning system
export interface LearningItemContent {
    mainEditor?: string;      // Content for main editor
    macroEditor?: string;     // Content for macro editor
    assemblyEditor?: string;  // Content for assembly editor
}

export interface EditorConfig {
    showMainEditor?: boolean;
    showMacroEditor?: boolean;
    mainEditorMode?: 'brainfuck' | 'assembly';
}

export interface InterpreterConfig {
    tapeSize?: number;        // Required tape size
    cellSize?: 256 | 65536 | 4294967296;  // 8-bit, 16-bit, or 32-bit
}

export interface DebuggerConfig {
    viewMode?: 'normal' | 'compact' | 'lane';
    laneCount?: number;       // Number of lanes for lane view
}

export interface TapeLabels {
    lanes?: { [key: number]: string };    // Lane labels (for lane view)
    columns?: { [key: number]: string };  // Column/word labels
    cells?: { [key: number]: string };    // Individual cell labels
}

export interface LearningItem {
    id: string;
    name: string;
    description: string;
    editorConfig: EditorConfig;
    interpreterConfig?: InterpreterConfig;
    debuggerConfig?: DebuggerConfig;
    labels?: TapeLabels;
    content: LearningItemContent;
}

export interface LearningSubcategory {
    id: string;
    name: string;
    items: LearningItem[];
}

export interface LearningCategory {
    id: string;
    name: string;
    icon?: string; // Optional emoji or icon identifier
    subcategories: LearningSubcategory[];
}

interface LearningState {
    categories: LearningCategory[];
    selectedItem: LearningItem | null;
}

class LearningStore {
    // Initialize with default learning content
    private defaultCategories: LearningCategory[] = [
        {
            id: 'ide',
            name: 'IDE',
            icon: '💻',
            subcategories: [
                {
                    id: 'ide-basics',
                    name: 'Getting Started',
                    items: [
                        {
                            id: 'ide-welcome',
                            name: 'Welcome to the IDE',
                            description: 'Introduction to the Brainfuck IDE',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: ideWelcome
                            }
                        },
                        {
                            id: 'ide-editors',
                            name: 'Working with Editors',
                            description: 'Learn about the main and macro editors',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: true,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: '// This is the main editor\n// The expanded macro code will appear here\n// Click "Expand Macros" button to see the result',
                                macroEditor: ideEditorsMacro
                            }
                        }
                    ]
                },
                {
                    id: 'ide-features',
                    name: 'IDE Features',
                    items: [
                        {
                            id: 'ide-debugging',
                            name: 'Debugging Basics',
                            description: 'Learn to use breakpoints and step through code',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 100,
                                cellSize: 256
                            },
                            debuggerConfig: {
                                viewMode: 'normal'
                            },
                            content: {
                                mainEditor: `// Debugging Tutorial
// Click on line numbers to set breakpoints!
// Try setting a breakpoint on line 10

// Initialize cell 0 with value 10
++++++++++

// Copy to cell 1 using a loop
[->+<]

// Move to cell 1 and add 5
>+++++

// Print the result (ASCII 15 = non-printable, so let's add more)
// Add 50 to make it printable
++++++++++ ++++++++++ ++++++++++ ++++++++++ ++++++++++

// Print the character
.

// Clear the cell
[-]

// Print newline
<++++++++++.`
                            },
                            labels: {
                                cells: {
                                    0: 'Source',
                                    1: 'Destination'
                                }
                            }
                        },
                        {
                            id: 'ide-tape-visualization',
                            name: 'Tape Visualization Modes',
                            description: 'Explore different tape view modes',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 256,
                                cellSize: 256
                            },
                            debuggerConfig: {
                                viewMode: 'lane',
                                laneCount: 8
                            },
                            content: {
                                mainEditor: `// Tape Visualization Demo
// Try switching between Normal, Compact, and Lane views!
// This creates a pattern that looks interesting in lane view

// Create a pattern across 64 cells
// Each group of 8 cells will form one row in lane view

// Row 1: Ascending values
>+>++>+++>++++>+++++>++++++>+++++++>++++++++

// Row 2: Descending values
>++++++++>+++++++>++++++>+++++>++++>+++>++>+

// Row 3: Even numbers
>++>++++>++++++>++++++++>++++++++++>++++++++++++>++++++++++++++>++++++++++++++++

// Row 4: Powers of 2
>+>++>++++>++++++++>++++++++++++++++>++++++++++++++++++++++++++++++++

// Move back to start
<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<

// Now step through or run to see the pattern!`
                            },
                            labels: {
                                lanes: {
                                    0: 'Pattern A',
                                    1: 'Pattern B',
                                    2: 'Pattern C',
                                    3: 'Pattern D',
                                    4: 'Pattern E',
                                    5: 'Pattern F',
                                    6: 'Pattern G',
                                    7: 'Pattern H'
                                }
                            }
                        },
                        {
                            id: 'ide-snapshots',
                            name: 'Using Snapshots',
                            description: 'Save and restore tape states',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 100,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: `// Snapshot Tutorial
// Run this program step by step and save snapshots at interesting points!

// Stage 1: Initialize some values
+++++ +++++ [>+++++ ++<-]  // Cell 1 = 70 (F)
>>.                         // Print F
>+++++ +++++ [>+++++ +++<-] // Cell 2 = 80 (P)  
>.                          // Print P

// SAVE A SNAPSHOT HERE! (Open Snapshots panel)

<<[-]>[-]>[-]               // Clear all cells

// Stage 2: Different values
<<<+++++ +++++ [>+++++ ++++<-]  // Cell 1 = 90 (Z)
>>.                              // Print Z
>+++++ ++[>+++++ +++++<-]        // Cell 2 = 65 (A)
>.                               // Print A

// SAVE ANOTHER SNAPSHOT HERE!

// Now try loading your first snapshot to restore the FP state!`
                            },
                            labels: {
                                cells: {
                                    0: 'Counter',
                                    1: 'Letter 1',
                                    2: 'Letter 2'
                                }
                            }
                        },
                        {
                            id: 'ide-execution-modes',
                            name: 'Execution Modes',
                            description: 'Compare different execution speeds',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 1000,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: `// Execution Modes Demo
// Try running this with different modes:
// 1. Step mode (Forward button) - see each operation
// 2. Smooth mode (Play button) - balanced speed
// 3. Turbo mode (Lightning button) - maximum speed
// 4. Custom delay (Clock button) - set your own speed
// 5. Rust WASM (Rocket button) - native speed!

// This program counts to 255 and shows progress
// The different modes will show dramatically different speeds!

// Initialize counter display
+++++ +++++ [>+++++ +++<-] >++. // Print 'R'
+++.                             // 'U'
-.                               // 'N'
<+++++ +++++.                    // newline

// Main counting loop
[-]  // Clear cell
[    // This will run 256 times
    + // Increment counter
    
    // Show progress every 10 increments
    >[-]+++++ +++++
    [<->-[<->-[<->-[<->-[<->-[<->-[<->-[<->-[<->-[<[-]>-]]]]]]]]]]
    <[>>+++++ +++++ [>+++++ ++<-]>.<<<<]  // Print dot for progress
    >>[-]<<
    
    // Continue until we overflow (256 becomes 0)
    +
]

// Done!
>+++++ +++++.  // newline
+++++ +++++ [>+++++ ++++<-]>.  // 'D'
+++++ +++++ [>+++++ +++++<-]>++++.  // 'O'
-------.  // 'N'
-.  // 'E'`
                            }
                        }
                    ]
                }
            ]
        },
        {
            id: 'brainfuck',
            name: 'Brainfuck',
            icon: '🧠',
            subcategories: [
                {
                    id: 'bf-basics',
                    name: 'Basics',
                    items: [
                        {
                            id: 'bf-hello',
                            name: 'Hello World',
                            description: 'Your first Brainfuck program',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: bfHelloWorld
                            },
                            labels: {
                                cells: {
                                    0: 'Temp',
                                    1: 'Letter'
                                }
                            }
                        },
                        {
                            id: 'bf-commands',
                            name: 'Basic Commands',
                            description: 'Learn the 8 Brainfuck commands',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: bfCommands
                            }
                        },
                        {
                            id: 'bf-loops',
                            name: 'Working with Loops',
                            description: 'Master the loop structure',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: bfLoops
                            },
                            labels: {
                                cells: {
                                    0: 'Initial Value',
                                    1: 'Copy Target',
                                    2: 'Temp Variable'
                                }
                            }
                        }
                    ]
                },
                {
                    id: 'bf-examples',
                    name: 'Examples',
                    items: [
                        {
                            id: 'bf-sierpinski',
                            name: 'Sierpinski Triangle',
                            description: 'Generate the Sierpinski triangle fractal',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: bfSierpinski
                            }
                        },
                        {
                            id: 'bf-mandelbrot',
                            name: 'Mandelbrot Fractal',
                            description: 'A complex fractal viewer by Erik Bosman',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000,
                                cellSize: 256
                            },
                            content: {
                                mainEditor: bfMandelbrot
                            }
                        }
                    ]
                }
            ]
        },
        {
            id: 'macro',
            name: 'Macro Language',
            icon: '🔧',
            subcategories: [
                {
                    id: 'macro-basics',
                    name: 'Basics',
                    items: [
                        {
                            id: 'macro-intro',
                            name: 'Introduction to Macros',
                            description: 'Learn how to define and use macros',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: true,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000,
                                cellSize: 256
                            },
                            content: {
                                macroEditor: macroIntro,
                                mainEditor: '// Expanded code will appear here after clicking "Expand Macros"'
                            }
                        },
                    ]
                },
                {
                    id: 'macro-advanced',
                    name: 'Advanced',
                    items: [
                        {
                            id: 'macro-ripple',
                            name: 'Ripple Hello World',
                            description: 'A complex macro example. Reeeeaaally complex.',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: true,
                                mainEditorMode: 'brainfuck'
                            },
                            interpreterConfig: {
                                tapeSize: 30000000,
                                cellSize: 65536
                            },
                            content: {
                                macroEditor: macroRipple,
                                mainEditor: '// Expanded code will appear here after clicking "Expand Macros"'
                            }
                        }
                    ]
                }
            ]
        }
    ];

    public state = new BehaviorSubject<LearningState>({
        categories: this.defaultCategories,
        selectedItem: null
    });

    // Select a learning item
    selectItem(item: LearningItem | null) {
        this.state.next({
            ...this.state.value,
            selectedItem: item
        });
    }


}

export const learningStore = new LearningStore();