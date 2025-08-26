import { BehaviorSubject } from 'rxjs';

// Import learning content files using Vite's ?raw suffix
// IDE content
import ideWelcome from '../learning-content/ide/basics/welcome.bf?raw';
import ideEditorsMacro from '../learning-content/ide/basics/editors-macro.bfm?raw';

// Brainfuck content
import bfHelloWorld from '../learning-content/brainfuck/basics/hello-world.bf?raw';
import bfCommands from '../learning-content/brainfuck/basics/commands.bf?raw';
import bfLoops from '../learning-content/brainfuck/basics/loops.bf?raw';
import bfMandelbrot from '../learning-content/brainfuck/examples/mandelbrot.bf?raw';

// Macro content
import macroIntro from '../learning-content/macro/basics/intro.bfm?raw';

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

export interface LearningItem {
    id: string;
    name: string;
    description: string;
    editorConfig: EditorConfig;
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
                            content: {
                                mainEditor: '// This is the main editor\n// The expanded macro code will appear here\n// Click "Expand Macros" button to see the result',
                                macroEditor: ideEditorsMacro
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
                            content: {
                                mainEditor: bfHelloWorld
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
                            content: {
                                mainEditor: bfLoops
                            }
                        }
                    ]
                },
                {
                    id: 'bf-examples',
                    name: 'Examples',
                    items: [
                        {
                            id: 'bf-mandelbrot',
                            name: 'Mandelbrot Fractal',
                            description: 'A complex fractal viewer by Erik Bosman',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
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