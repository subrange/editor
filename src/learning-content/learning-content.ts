import type {LearningCategory} from "../stores/learning.store.ts";

// IDE content
import ideWelcome from './ide/basics/welcome.bf?raw';
import ideEditorsMacro from './ide/basics/editors-macro.bfm?raw';
import ideDebugging from './ide/features/debugging.bf?raw';
import ideTapeVisualization from './ide/features/tape-visualization.bf?raw';
import ideSnapshots from './ide/features/snapshots.bf?raw';
import ideExecutionModes from './ide/features/execution-modes.bf?raw';

// Brainfuck content
import bfHelloWorld from './brainfuck/basics/hello-world.bf?raw';
import bfCommands from './brainfuck/basics/commands.bf?raw';
import bfLoops from './brainfuck/basics/loops.bf?raw';
import bfMandelbrot from './brainfuck/examples/mandelbrot.bf?raw';
import bfSierpinski from './brainfuck/examples/sierpinski.bf?raw';
import bfYapi from './brainfuck/examples/yapi.bf?raw';
import bfChar from './brainfuck/examples/char.bf?raw';

// Macro content
import macroIntro from './macro/basics/intro.bfm?raw';
import macroRipple from './macro/advanced/ripple-hello-world.bfm?raw';

export const learningContent: LearningCategory[] = [
    {
        id: 'ide',
        name: 'Braintease IDE',
        icon: '💻',
        subcategories: [
            {
                id: 'ide-basics',
                name: 'Getting Started',
                items: [
                    {
                        id: 'ide-welcome',
                        name: 'Welcome to the IDE',
                        description: 'Introduction to the Braintease IDE',
                        editorConfig: {
                            showMainEditor: true,
                            showMacroEditor: false,
                            mainEditorMode: 'brainfuck'
                        },
                        interpreterConfig: {
                            tapeSize: 30000,
                            cellSize: 256
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
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
                        debuggerConfig: {
                            viewMode: 'normal'
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
                            tapeSize: 30000,
                            cellSize: 256
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
                        },
                        content: {
                            mainEditor: ideDebugging,
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
                            tapeSize: 30000,
                            cellSize: 256
                        },
                        debuggerConfig: {
                            viewMode: 'lane',
                            laneCount: 8
                        },
                        content: {
                            mainEditor: ideTapeVisualization,
                        },
                        labels: {
                            lanes: {
                                0: 'A',
                                1: 'B',
                                2: 'C',
                                3: 'D',
                                4: 'E',
                                5: 'F',
                                6: 'G',
                                7: 'H'
                            },
                            columns: {
                                0: 'Asc',
                                1: 'Desc',
                                2: 'Even',
                                3: 'Powers'
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
                            tapeSize: 30000,
                            cellSize: 256
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
                        },
                        content: {
                            mainEditor: ideSnapshots,
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
                            tapeSize: 30000,
                            cellSize: 256
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
                        },
                        content: {
                            mainEditor: ideExecutionModes,
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
                        debuggerConfig: {
                            viewMode: 'normal'
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
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
                        },
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
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
                        },
                    }
                ]
            },
            {
                id: 'bf-examples',
                name: 'Examples',
                items: [
                    {
                        id: 'bf-char',
                        name: 'Characters',
                        description: 'Print ASCII characters set',
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
                            mainEditor: bfChar
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
                        },
                    },
                    {
                        id: 'bf-yapi',
                        name: 'YAPI',
                        description: 'A Brainfuck program that calculates Pi',
                        editorConfig: {
                            showMainEditor: true,
                            showMacroEditor: false,
                            mainEditorMode: 'brainfuck'
                        },
                        interpreterConfig: {
                            tapeSize: 30000,
                            cellSize: 65536
                        },
                        content: {
                            mainEditor: bfYapi
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
                        }
                    },
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
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
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
                        debuggerConfig: {
                            viewMode: 'normal'
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
                        },
                        debuggerConfig: {
                            viewMode: 'normal'
                        },
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
                            tapeSize: 300_000_000,
                            cellSize: 65536
                        },
                        content: {
                            macroEditor: macroRipple,
                            mainEditor: '// Expanded code will appear here after clicking "Expand Macros"'
                        },
                        debuggerConfig: {
                            viewMode: 'lane',
                            laneCount: 8,
                        },
                        labels: {
                            lanes: {
                                0: 'ZERO',
                                1: 'REG',
                                2: 'PTR',
                                3: 'CODE',
                                4: 'RAM',
                                5: 'Sa',
                                6: 'Sb',
                                7: 'FL'
                            },
                            columns: {
                                0: 'Zero',
                                1: 'R0',
                                2: 'PC',
                                3: 'PCB',
                                4: 'RA',
                                5: 'RAB',
                            },
                            cells: {
                                0: 'State',
                                9: 'R0',
                                17: 'PC',
                                25: 'PCB',
                                33: 'RA',
                                41: 'RAB',
                                49: 'RV0',
                                57: 'RV1',

                                65: 'A0',
                                73: 'A1',
                                81: 'A2',
                                89: 'A3',

                                97: 'X0',
                                105: 'X1',
                                113: 'X2',
                                121: 'X3',

                                129: 'T0',
                                137: 'T1',
                                145: 'T2',
                                153: 'T3',
                                161: 'T4',
                                169: 'T5',
                                177: 'T6',
                                185: 'T7',

                                193: 'S0',
                                201: 'S1',
                                209: 'S2',
                                217: 'S3',

                                225: 'SC',
                                233: 'SB',
                                241: 'SP',
                                249: 'FP',
                                257: 'GP',


                                299: 'Ins 0'
                            }
                        }
                    }
                ]
            }
        ]
    }
];