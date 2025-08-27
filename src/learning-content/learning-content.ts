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
                    },
                    {
                        "id": "bf-echo",
                        "name": "Reading Input",
                        "description": "Echo input until Enter",
                        "editorConfig": {
                            "showMainEditor": true,
                            "showMacroEditor": false,
                            "mainEditorMode": "brainfuck"
                        },
                        "interpreterConfig": {
                            "tapeSize": 30000,
                            "cellSize": 256
                        },
                        "debuggerConfig": {
                            "viewMode": "normal"
                        },
                        "content": {
                            "mainEditor": "[\n  // This program reads the letter from an input, echoes it, and does it until you press enter.\n  // Run it with the lighting or play button\n]\n----------[++++++++++>,.----------]++++++++++",
                            "macroEditor": ""
                        }
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
                            viewMode: 'lane',
                            laneCount: 2
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
                    {
                        "id": "macro-lanes",
                        "name": "Lanes and Words",
                        "description": "Using lanes and words to simplify Brainfuck code",
                        "editorConfig": {
                            "showMainEditor": true,
                            "showMacroEditor": true,
                            "mainEditorMode": "brainfuck"
                        },
                        "interpreterConfig": {
                            "tapeSize": 30000,
                            "cellSize": 256
                        },
                        "debuggerConfig": {
                            "viewMode": "lane",
                            "laneCount": 3
                        },
                        "content": {
                            "mainEditor": ">>> [ Moves us to the next word, to cell number 3 ]\n<<< [ Gets us back to cell 0 ]\n> [ Moved to lane 1, basically, to cell 1 ]\n+ [ - Added (and immediately removed) one ]\n< [ Moved back to lane 0 ]\n>>> [ Moved to the next word, lane 0]\n<<< [ And got back to word 0 ]\n>+++< [ Move to lane 1, add 3, return to lane 0 ]\n>>>\n>+++<   [ Move to lane 1, add 3, return to lane 0 ]\n>>+++++<< [ Move to lane 2, add 5, return to lane 0 ]\n>>>\n[So now let's add 3 and 5 using lanes one and two]\n>+++<\n>>+++++<<\n>>[-<+>]<<\n>>>\n[ Now let's do the same but on one lane]\n>+++>>>+++++<<<>>>[-<<<+>>>]<<<<",
                            "macroEditor": "/*\n\nThis is a tutorial on using \"lanes\" and \"words\" abstractions.\nIt is much easier to write complex Brainfuck programs, if you look\nat Brainfuck tape as not a single continuous tape, but as\non multiple interleaved \"lanes\".\n\nThis technique was previously used, for example, in the\nSierpinski triange implementation by Daniel Cristofani.\n\nImagine Brainfuck tape:\n\n[0, 1, 2, 3, 4, 5, 6, 7, 8...]\n\nDefine how many lanes you need. This example uses three,\nmy Ripple VM implementation uses eight. The amount of lanes\ndefines your word size. Let's define it as LANES_COUNT\n\n*/\n\n#define LANES_COUNT 3\n\n/*\n\nFor three lanes:\n\n[[0, 1, 2], [3, 4, 5], [6, 7, 8]...]\n\nNow we can say that word 0 is cells 0, 1, 2, and so on.\nAnd now, to see the whole picture, let's write our words\nin columns:\n\n0 | 3 | 6\n1 | 4 | 7\n2 | 5 | 8\n\nNow we can say that 0 | 3 | 6 are laying on lane 0,\n1 | 4 | 7 are on lane 1, and\n2 | 5 | 8 are on lane 2.\n\nRemember, we still have the same flat Brainfuck tape —\nwe just think about it differently.\n\nSo now we can actually think about \"moving to the next word\non the same lane\". This would be just using > LANES_COUNT\ntimes, right? Same with \"moving to the previous word on\nthe same lane\" — it is the same as using < LANES_COUNT times.\n\nLet's define these and check them:\n*/\n\n#define right(n) {repeat(n, >)}\n#define left(n)  {repeat(n, <)}\n\n#define nextword @right(#LANES_COUNT)\n#define prevword @left( #LANES_COUNT)\n\n/* And now, make sure you also look in the main editor */\n\n@nextword [ Moves us to the next word, to cell number 3 ]\n@prevword [ Gets us back to cell 0 ]\n\n/*\n\nNow let's think about how we can move between lanes.\nIf we are on the lane 0, we can move to lane 1 by just >,\nand back by <. From lane 0 to lane 2 it would be >>.\n\nSo to move to lane N from lane 0, we need to > N times,\nwhere N is a number of lane!\n\nLet's look at this:\n*/\n\n> [ Moved to lane 1, basically, to cell 1 ]\n+ [ - Added (and immediately removed) one ]\n< [ Moved back to lane 0 ]\n\n@nextword [ Moved to the next word, lane 0]\n@prevword [ And got back to word 0 ]\n\n/*\n\nPerfect! The only thing is that is not, is that we now\nneed to be very cautious on which lane we are at. It\nis pretty easy to lose track of it, especially when\nyou write a large program.\n\nWhile writing the whole virtual machine, to simplify things\nfor myself, I started thinking about \"lane hygiene\". I\ndefined lane 0 as my \"homebase\", and made sure that\nafter every complex operation on other lanes, I must\nreturn to lane 0. And I defined a macro to help me with it:\n*/\n\n#define LANE_ZERO 0 // Technically unused in this example\n#define LANE_ONE  1\n#define LANE_TWO  2\n\n#define lane(n, code) {\n  @right(n) // Move to lane n\n  code      // Do stuff\n  @left(n)  // Return to lane 0\n}\n\n@lane(#LANE_ONE, +++) [ Move to lane 1, add 3, return to lane 0 ]\n\n@nextword\n\n@lane(#LANE_ONE, +++)   [ Move to lane 1, add 3, return to lane 0 ]\n@lane(#LANE_TWO, +++++) [ Move to lane 2, add 5, return to lane 0 ]\n\n@nextword\n\n// MARK: Addition of values in the same word, but on different lanes\n\n[So now let's add 3 and 5 using lanes one and two]\n\n@lane(#LANE_ONE, +++)\n@lane(#LANE_TWO, +++++)\n\n@lane(#LANE_ONE,\n  > // move to lane two\n  [-<+>] // move the value from lane two to lane one\n  < // return to the lane one so @lane macro would quit to lane zero\n)\n\n@nextword\n\n// MARK: Addition of values in the same lane, but on different words\n\n[ Now let's do the same but on one lane]\n\n@lane(#LANE_ONE,\n  +++ @nextword +++++ @prevword\n  \n  @nextword // Move to the next word in the same lane\n  [- @prevword + @nextword] // move the value from this word to the previous one\n  @prevword // Return to the previous word\n)\n\n/*\n\nYou see, how simple it is? The beauty is that every Brainfuck\nalgorithm can be executed with this abstraction —\njust replace every > with @nextword, and every < with @prevword.\n\nIt is like having multiple Brainfuck tapes — but still\non one master tape.\n\nI made the lane visualizer in this IDE specifically to\nallow me to develop the VM easier.\n*/"
                        },
                        labels: {
                            lanes: {
                                0: 'Lane 0',
                                1: 'Lane 1',
                                2: 'Lane 2'
                            },
                            columns: {
                                0: 'Word 0',
                                1: 'Word 1',
                                2: 'Word 2',
                                3: 'Word 3',
                                4: 'Word 4',
                                5: 'Word 5',
                            },
                        }
                    }
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