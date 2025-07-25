// @ts-ignore
import init, { BrainfuckInterpreter } from '../wasm/rust_bf';

let interpreter: BrainfuckInterpreter | null = null;
let wasmInitialized = false;

interface WorkerMessage {
    type: 'init' | 'setCode' | 'step' | 'runTurbo' | 'pause' | 'resume' | 'stop' | 
          'toggleBreakpoint' | 'clearBreakpoints' | 'getState' | 'getTapeSlice' |
          'setTapeSize' | 'setCellSize' | 'setLaneCount' | 'reset';
    data?: unknown;
}

interface WorkerResponse {
    type: string;
    data?: unknown;
    error?: string;
}

async function initializeWasm() {
    if (!wasmInitialized) {
        await init();
        wasmInitialized = true;
    }
}

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
    const { type, data } = event.data;
    let response: WorkerResponse;

    try {
        switch (type) {
            case 'init': {
                await initializeWasm();
                const { tapeSize, cellSize } = data as { tapeSize: number; cellSize: number };
                interpreter = new BrainfuckInterpreter(tapeSize, cellSize);
                // Enable unsafe mode for maximum performance (like El Brainfuck's "undefined" mode)
                interpreter.set_unsafe_mode(true);
                response = { type: 'initialized' };
                break;
            }

            case 'setCode': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                interpreter.set_code(JSON.stringify((data as { code: unknown }).code));
                response = { type: 'codeSet' };
                break;
            }

            case 'step': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                const hasMore = interpreter.step();
                const state = interpreter.get_state();
                const stateObj = JSON.parse(state);
                
                // Get current position
                try {
                    const positionJson = interpreter.get_current_position();
                    const position = JSON.parse(positionJson);
                    stateObj.currentPosition = position;
                } catch (e) {
                    console.error('Failed to get current position:', e);
                }
                
                console.log(`Step result: hasMore=${hasMore}, isPaused=${stateObj.is_paused}, isStopped=${stateObj.is_stopped}`);
                
                // Check if the interpreter has paused (e.g., due to a breakpoint)
                if (stateObj.is_paused) {
                    response = { type: 'paused', data: { state: JSON.stringify(stateObj) } };
                } else {
                    response = { type: 'stepped', data: { hasMore, state: JSON.stringify(stateObj) } };
                }
                break;
            }

            case 'runTurbo': {
                if (!interpreter) throw new Error('Interpreter not initialized');

                interpreter.set_unsafe_mode(true);

                // Run turbo execution in batches for streaming updates
                const BATCH_SIZE = 200_000_000; // 100 million operations per batch
                const UPDATE_INTERVAL = 300; // Send updates every 100ms
                
                const runBatched = async () => {
                    let lastUpdateTime = Date.now();
                    let totalOps = 0;
                    
                    try {
                        while (true) {
                            const hasMore = interpreter!.run_turbo_batch(BATCH_SIZE);
                            totalOps += BATCH_SIZE;
                            
                            const now = Date.now();
                            if (now - lastUpdateTime >= UPDATE_INTERVAL || !hasMore) {
                                // Send progress update
                                const state = interpreter!.get_state();
                                const stateObj = JSON.parse(state);
                                
                                // Get current position
                                try {
                                    const positionJson = interpreter!.get_current_position();
                                    const position = JSON.parse(positionJson);
                                    stateObj.currentPosition = position;
                                } catch (e) {
                                    // Ignore position errors during execution
                                }
                                
                                self.postMessage({ 
                                    type: hasMore ? 'turboProgress' : 'turboComplete',
                                    data: { 
                                        state: JSON.stringify(stateObj),
                                        iterations: totalOps
                                    } 
                                });
                                
                                lastUpdateTime = now;
                            }
                            
                            if (!hasMore) {
                                break;
                            }
                            
                            // Yield to allow other messages to be processed
                            await new Promise(resolve => setTimeout(resolve, 0));
                        }
                    } catch (error) {
                        self.postMessage({
                            type: 'error',
                            error: error instanceof Error ? error.message : 'Turbo execution failed'
                        });
                    }
                };
                
                setTimeout(runBatched, 0);
                response = { type: 'turboStarted' };
                break;
            }

            case 'pause': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                interpreter.pause();
                response = { type: 'paused' };
                break;
            }

            case 'resume': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                console.log('Worker: Calling interpreter.resume()');
                interpreter.resume();
                const state = interpreter.get_state();
                const stateObj = JSON.parse(state);
                console.log(`Worker: After resume - isPaused=${stateObj.is_paused}`);
                response = { type: 'resumed' };
                break;
            }

            case 'stop': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                interpreter.stop();
                response = { type: 'stopped' };
                break;
            }

            case 'toggleBreakpoint': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                const { line, column } = data as { line: number; column: number };
                interpreter.toggle_breakpoint(line, column);
                response = { type: 'breakpointToggled' };
                break;
            }

            case 'clearBreakpoints': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                interpreter.clear_breakpoints();
                response = { type: 'breakpointsCleared' };
                break;
            }

            case 'getState': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                const stateJson = interpreter.get_state();
                response = { type: 'state', data: JSON.parse(stateJson) };
                break;
            }

            case 'getTapeSlice': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                const { start, end } = data as { start: number; end: number };
                const slice = interpreter.get_tape_slice(start, end);
                response = { type: 'tapeSlice', data: { start, slice } };
                break;
            }

            case 'setTapeSize': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                interpreter.set_tape_size((data as { size: number }).size);
                response = { type: 'tapeSizeSet' };
                break;
            }

            case 'setCellSize': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                interpreter.set_cell_size((data as { size: number }).size);
                response = { type: 'cellSizeSet' };
                break;
            }

            case 'setLaneCount': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                interpreter.set_lane_count((data as { count: number }).count);
                response = { type: 'laneCountSet' };
                break;
            }

            case 'reset': {
                if (!interpreter) throw new Error('Interpreter not initialized');
                interpreter.reset();
                response = { type: 'reset' };
                break;
            }

            default:
                throw new Error(`Unknown message type: ${type}`);
        }
        
        self.postMessage(response);
    } catch (error) {
        self.postMessage({
            type: 'error',
            error: error instanceof Error ? error.message : String(error)
        });
    }
};