// Modified brainfuck interpreter with loop support

import {BehaviorSubject} from "rxjs";
import {editorStore, type Line, type Position} from "../editor/editor.store.ts";

type InterpreterState = {
    tape: Uint8Array;
    pointer: number;
    isRunning: boolean;

    output: string;
}

const TAPE_SIZE = 1024 * 1024; // 1 megabyte tape

class InterpreterStore {
    public state = new BehaviorSubject<InterpreterState>({
        tape: new Uint8Array(TAPE_SIZE).fill(0),
        pointer: 0,
        isRunning: false,
        output: ''
    })

    private code: Array<Line> = [];

    public currentChar = new BehaviorSubject<Position>({
        line: 0,
        column: 0
    })

    private loopMap: Map<string, Position> = new Map();
    private runInterval: number | null = null;
    private runAnimationFrameId: number | null = null;

    constructor() {
        // Sync the code with the editor store
        editorStore.editorState.subscribe(s => {
            if (JSON.stringify(s.lines) !== JSON.stringify(this.code)) { // Yep, I do not care about performance here
                this.reset();
                this.code = s.lines;
                this.buildLoopMap();
            }
        });
    }

    public reset() {
        this.state.next({
            tape: new Uint8Array(TAPE_SIZE).fill(0),
            pointer: 0,
            isRunning: false,
            output: ''
        });
        this.currentChar.next({
            line: 0,
            column: 0
        });
        if (this.runInterval) {
            clearInterval(this.runInterval);
            this.runInterval = null;
        }

        if (this.runAnimationFrameId) {
            cancelAnimationFrame(this.runAnimationFrameId);
            this.runAnimationFrameId = null;
        }
    }

    // Build a map of matching brackets for efficient jumping
    private buildLoopMap() {
        this.loopMap.clear();
        const stack: Position[] = [];

        for (let line = 0; line < this.code.length; line++) {
            const text = this.code[line].text;
            for (let column = 0; column < text.length; column++) {
                const char = text[column];
                const pos = { line, column };

                if (char === '[') {
                    stack.push(pos);
                } else if (char === ']') {
                    if (stack.length === 0) {
                        console.error(`Unmatched ] at line ${line}, column ${column}`);
                        continue;
                    }
                    const openPos = stack.pop()!;
                    // Map both directions
                    this.loopMap.set(this.posToKey(openPos), pos);
                    this.loopMap.set(this.posToKey(pos), openPos);
                }
            }
        }

        if (stack.length > 0) {
            console.error(`Unmatched [ brackets:`, stack);
        }
    }

    private posToKey(pos: Position): string {
        return `${pos.line},${pos.column}`;
    }

    private moveToNextChar() {
        const current = this.currentChar.getValue();

        if (current.column < this.code[current.line].text.length - 1) {
            // Move to next column
            this.currentChar.next({
                line: current.line,
                column: current.column + 1
            });
        } else if (current.line < this.code.length - 1) {
            // Move to next line
            this.currentChar.next({
                line: current.line + 1,
                column: 0
            });
        } else {
            // End of code
            return false;
        }
        return true;
    }

    private moveToNextLine() {
        const current = this.currentChar.getValue();

        if (current.line < this.code.length - 1) {
            // Move to next line
            this.currentChar.next({
                line: current.line + 1,
                column: 0
            });
            return true;
        } else {
            // End of code
            return false;
        }
    }

    private getCharAt(pos: Position): string | null {
        if (pos.line >= this.code.length) return null;
        const line = this.code[pos.line];
        if (pos.column >= line.text.length) return null;
        return line.text[pos.column];
    }

    private getCurrentChar(): string | null {
        return this.getCharAt(this.currentChar.getValue());
    }

    public step(): boolean {
        const currentState = {
            ...this.state.getValue()
        };

        const char = this.getCurrentChar();
        const currentPos = this.currentChar.getValue();

        if (char === '/') {
            const hasMore = this.moveToNextLine();
            if (!hasMore) {
                console.log("Program finished.");
                this.stop();
                return false;
            }
            // Continue processing from the new line
            return this.step();
        }

        // Skip non-command characters
        if (char === null || (char && !'><+-[].,'.includes(char))) {
            const hasMore = this.moveToNextChar();
            if (!hasMore) {
                console.log("Program finished.");
                this.stop(); // Use stop() method to handle cleanup
                return false;
            }
            // If we just skipped a non-command character, try stepping again
            return this.step();
        }

        let shouldMoveNext = true;

        switch (char) {
            case '>':
                currentState.pointer = (currentState.pointer + 1) % currentState.tape.length;
                break;
            case '<':
                currentState.pointer = (currentState.pointer - 1 + currentState.tape.length) % currentState.tape.length;
                break;
            case '+':
                currentState.tape[currentState.pointer] = (currentState.tape[currentState.pointer] + 1) % 256;
                break;
            case '-':
                currentState.tape[currentState.pointer] = (currentState.tape[currentState.pointer] - 1 + 256) % 256;
                console.log(currentState.tape)
                break;
            case '[':
                // If current cell is 0, jump to matching ]
                if (currentState.tape[currentState.pointer] === 0) {
                    const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                    if (matchingPos) {
                        this.currentChar.next(matchingPos);
                        shouldMoveNext = true; // Will move past the ]
                    } else {
                        console.error(`No matching ] for [ at ${currentPos.line}:${currentPos.column}`);
                    }
                }
                break;
            case ']':
                // If current cell is not 0, jump back to matching [
                if (currentState.tape[currentState.pointer] !== 0) {
                    const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                    if (matchingPos) {
                        this.currentChar.next(matchingPos);
                        shouldMoveNext = true; // Will move past the [
                    } else {
                        console.error(`No matching [ for ] at ${currentPos.line}:${currentPos.column}`);
                    }
                }
                break;
            case '.':
                // console.log(`Output: ${String.fromCharCode(currentState.tape[currentState.pointer])}`);
                currentState.output += String.fromCharCode(currentState.tape[currentState.pointer]);
                break;
            case ',':
                console.log(`Input requested at position ${currentState.pointer}`);
                break;
        }

        this.state.next(currentState);

        if (shouldMoveNext) {
            const hasMore = this.moveToNextChar();
            if (!hasMore) {
                console.log("Program finished.");
                this.stop(); // Use stop() method to handle cleanup
                return false;
            }
        }

        return true;
    }

    public run(delay: number = 100) {
        if (this.runInterval) {
            clearInterval(this.runInterval);
        }

        this.state.next({
            ...this.state.getValue(),
            isRunning: true
        });

        this.runInterval = window.setInterval(() => {
            this.step();

            // Check if we've reached the end
            const current = this.currentChar.getValue();
            if (current.line >= this.code.length) {
                this.stop();
            }
        }, delay);
    }

    public runSmooth = () => {
        // Run with requestAnimationFrame for smooth execution
        this.state.next({
            ...this.state.getValue(),
            isRunning: true
        });

        const step = () => {
            const r = this.step();

            if (!r) {
                return;
            }

            // Check if we've reached the end
            const current = this.currentChar.getValue();
            if (current.line < this.code.length) {
                this.runAnimationFrameId = requestAnimationFrame(step);
            } else {
                this.stop();
            }
        };

        this.runAnimationFrameId = requestAnimationFrame(step);

        this.runInterval = null;
    }

    public async runImmediately() {
        this.state.next({
            ...this.state.getValue(),
            isRunning: true
        });

        while (this.step()) {
            const current = this.currentChar.getValue();
            if (current.line >= this.code.length) {
                this.stop();
                break;
            }
        }
    }

    public stop() {
        if (this.runInterval) {
            clearInterval(this.runInterval);
            this.runInterval = null;

            this.state.next({
                ...this.state.getValue(),
                isRunning: false
            });
        }

        if (this.runAnimationFrameId) {
            cancelAnimationFrame(this.runAnimationFrameId);
            this.runAnimationFrameId = null;
        }
    }
}

export const interpreterStore = new InterpreterStore();