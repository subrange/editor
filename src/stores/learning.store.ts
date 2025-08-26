import { BehaviorSubject } from 'rxjs';

// Import learning content files using Vite's ?raw suffix
// IDE content
import ideWelcome from '../learning-content/ide/basics/welcome.bf?raw';
import ideEditorsMacro from '../learning-content/ide/basics/editors-macro.bfm?raw';

// Brainfuck content
import bfHelloWorld from '../learning-content/brainfuck/basics/hello-world.bf?raw';
import bfCommands from '../learning-content/brainfuck/basics/commands.bf?raw';
import bfMandelbrot from '../learning-content/brainfuck/examples/mandelbrot.bf?raw';

// Macro content
import macroIntro from '../learning-content/macro/basics/intro.bfm?raw';

// RVM content
import rvmIntro from '../learning-content/rvm/basics/intro.asm?raw';

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
                                mainEditor: '// Loop examples in Brainfuck\n\n// Clear a cell (set to 0)\n[-]\n\n// Copy value from cell 0 to cell 1\n// Assumes cell 1 and 2 are zero\n[->+>+<<]>>[-<<+>>]<<\n\n// Multiply 3 * 5 = 15\n+++>+++++[<[->+>+<<]>>[-<<+>>]<-]<\n\n// Division with remainder\n// Divide 13 by 4: quotient in cell 1, remainder in cell 0\n+++++++++++++  // Set cell 0 to 13 (dividend)\n>>>>++++       // Set cell 4 to 4 (divisor)\n<<<<           // Back to cell 0\n[>>>>[-<<<<-<+>>>>>]<<<<[->>>>>+<<<<<]>-]  // Division loop\n// Cell 1 has quotient (3), cell 0 has remainder (1)'
                            }
                        }
                    ]
                },
                {
                    id: 'bf-examples',
                    name: 'Examples',
                    items: [
                        {
                            id: 'bf-fibonacci',
                            name: 'Fibonacci Sequence',
                            description: 'Generate Fibonacci numbers',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            content: {
                                mainEditor: '// Fibonacci sequence generator\n// Generates and prints first 10 Fibonacci numbers\n\n// Initialize\n++++++++++ // Counter in cell 0 (10 iterations)\n>+         // First Fib number in cell 1 (1)\n>          // Second Fib number in cell 2 (starts at 0)\n\n// Main loop\n[\n    // Print current number (simplified - prints symbols, not digits)\n    <.         // Print current Fibonacci number\n    \n    // Calculate next Fibonacci number\n    // Copy cell 1 to cells 2 and 3\n    [->+>+<<]  \n    \n    // Move cell 3 back to cell 1\n    >>[-<<+>>] \n    \n    // Cell 2 now has the sum (next Fib number)\n    // Swap cells 1 and 2 for next iteration\n    <[->+<]    \n    \n    // Decrement counter\n    <<-        \n]\n\n// Note: This prints ASCII characters, not decimal numbers\n// For readable output, you would need to implement decimal conversion'
                            }
                        },
                        {
                            id: 'bf-input-echo',
                            name: 'Input Echo',
                            description: 'Read and echo user input',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: false,
                                mainEditorMode: 'brainfuck'
                            },
                            content: {
                                mainEditor: '// Input Echo Program\n// Reads characters from input and echoes them back\n// Stops when user enters a null character (0)\n\n// Main loop: read and echo until null\n,          // Read first character\n[\n    .      // Output the character\n    ,      // Read next character\n]          // Loop while not null\n\n// Program ends when null character is received'
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
                        {
                            id: 'macro-functions',
                            name: 'Built-in Functions',
                            description: 'Use repeat, if, for, and more',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: true,
                                mainEditorMode: 'brainfuck'
                            },
                            content: {
                                macroEditor: '// Built-in functions provide powerful code generation\n\n// === REPEAT Function ===\n// Syntax: {repeat(count, code)}\n{repeat(5, +)}  // Generates: +++++\n{repeat(3, >)}  // Generates: >>>\n\n// === IF Function ===\n// Syntax: {if(condition, then_code, else_code)}\n// Note: Conditions are evaluated at expansion time\n#define DEBUG 1\n{if(DEBUG, [-]+++, )}  // If DEBUG, clear and add 3\n\n// === FOR Function ===\n// Syntax: {for(var, start, end, code)}\n// Generates unrolled loop\n{for(i, 0, 3, >+)}  // Generates: >+>+>+\n\n// === REVERSE Function ===\n// Reverses the order of commands\n{reverse(+->.<)}  // Generates: >.>-+\n\n// === Combining Functions ===\n#define set_ascii(c) [-]{repeat(c, +)}\n#define print_times(n, c) @set_ascii(c) {repeat(n, .)}\n\n// Print "AAA"\n@print_times(3, 65)\n\n// Advanced: Nested functions\n{repeat(2, {repeat(3, +)}>)}  // Generates: +++>+++>\n\n// Click "Expand Macros" to see the result!',
                                mainEditor: '// Expanded Brainfuck code will appear here'
                            }
                        }
                    ]
                },
                {
                    id: 'macro-advanced',
                    name: 'Advanced',
                    items: [
                        {
                            id: 'macro-recursion',
                            name: 'Recursive Macros',
                            description: 'Create powerful recursive patterns',
                            editorConfig: {
                                showMainEditor: true,
                                showMacroEditor: true,
                                mainEditorMode: 'brainfuck'
                            },
                            content: {
                                macroEditor: '// Recursive macro patterns\n// Note: Be careful with recursion depth!\n\n// Example: Power of 2 generator\n#define pow2(n) {if(n > 0, +@pow2(n - 1)@pow2(n - 1), +)}\n\n// @pow2(3) generates 2^3 = 8 plus signs\n@pow2(3)\n\n// Factorial-like repetition\n#define factorial_repeat(n, code) {if(n > 1, {repeat(n, code)}@factorial_repeat(n - 1, code), code)}\n\n// This creates a triangular pattern\n@factorial_repeat(4, +)  // 4+3+2+1 = 10 plus signs\n\n// Binary tree structure\n#define tree(depth) {if(depth > 0, >@tree(depth - 1)<@tree(depth - 1), +)}\n\n@tree(2)  // Creates a binary tree pattern in memory\n\n// Click "Expand Macros" to see the patterns!',
                                mainEditor: '// Recursive patterns will expand here'
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

    // Get all categories
    getCategories(): LearningCategory[] {
        return this.state.value.categories;
    }

    // Get a specific category by ID
    getCategory(categoryId: string): LearningCategory | undefined {
        return this.state.value.categories.find(c => c.id === categoryId);
    }

    // Get a specific item by IDs
    getItem(categoryId: string, subcategoryId: string, itemId: string): LearningItem | undefined {
        const category = this.getCategory(categoryId);
        if (!category) return undefined;

        const subcategory = category.subcategories.find(s => s.id === subcategoryId);
        if (!subcategory) return undefined;

        return subcategory.items.find(i => i.id === itemId);
    }

    // Select a learning item
    selectItem(item: LearningItem | null) {
        this.state.next({
            ...this.state.value,
            selectedItem: item
        });
    }

    // Add a new category
    addCategory(category: LearningCategory) {
        const categories = [...this.state.value.categories, category];
        this.state.next({
            ...this.state.value,
            categories
        });
    }

    // Update a category
    updateCategory(categoryId: string, updates: Partial<LearningCategory>) {
        const categories = this.state.value.categories.map(c =>
            c.id === categoryId ? { ...c, ...updates } : c
        );
        this.state.next({
            ...this.state.value,
            categories
        });
    }

    // Add item to a subcategory
    addItem(categoryId: string, subcategoryId: string, item: LearningItem) {
        const categories = this.state.value.categories.map(category => {
            if (category.id !== categoryId) return category;

            const subcategories = category.subcategories.map(subcategory => {
                if (subcategory.id !== subcategoryId) return subcategory;
                
                return {
                    ...subcategory,
                    items: [...subcategory.items, item]
                };
            });

            return { ...category, subcategories };
        });

        this.state.next({
            ...this.state.value,
            categories
        });
    }
}

export const learningStore = new LearningStore();