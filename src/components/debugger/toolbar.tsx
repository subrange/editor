import {
  PlayIcon,
  StopIcon,
  ArrowPathIcon,
  BoltIcon,
  ClockIcon,
  XMarkIcon,
  CursorArrowRaysIcon,
  RocketLaunchIcon,
} from '@heroicons/react/24/solid';
import { interpreterStore } from './interpreter-facade.store.ts';
import { useStoreSubscribe } from '../../hooks/use-store-subscribe.tsx';
import { useState, useEffect } from 'react';
import { ForwardIcon, PauseIcon } from '@heroicons/react/24/solid';
import { IconButton } from '../ui/icon-button.tsx';
import { editorManager } from '../../services/editor-manager.service.ts';
import { rustWasmInterpreter } from '../../services/rust-wasm-interpreter.service.ts';
import { outputStore } from '../../stores/output.store.ts';
import { settingsStore } from '../../stores/settings.store.ts';

export function Toolbar() {
  const interpreterState = useStoreSubscribe(interpreterStore.state);
  const {
    isRunning,
    isPaused,
    isStopped,
    isWaitingForInput,
    lastExecutionMode,
    lastExecutionTime,
    lastOperationCount,
  } = interpreterState;
  const [delay, setDelay] = useState(50);
  const [showDelayInput, setShowDelayInput] = useState(false);
  const [wasmStatus, setWasmStatus] = useState<
    'initializing' | 'ready' | 'error'
  >('initializing');
  const [wasmRunning, setWasmRunning] = useState(false);
  const [wasmWaitingForInput, setWasmWaitingForInput] = useState(false);

  useEffect(() => {
    const statusSub = rustWasmInterpreter.status$.subscribe(setWasmStatus);
    const runningSub = rustWasmInterpreter.isRunning$.subscribe(setWasmRunning);
    const waitingInputSub = rustWasmInterpreter.isWaitingForInput$.subscribe(
      setWasmWaitingForInput,
    );

    return () => {
      statusSub.unsubscribe();
      runningSub.unsubscribe();
      waitingInputSub.unsubscribe();
    };
  }, []);

  // Debug logging
  if (isStopped) {
    console.log(
      'Toolbar: Program stopped. Execution time:',
      lastExecutionTime,
      'Operation count:',
      lastOperationCount,
    );
  }

  const handleDelayChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseInt(e.target.value) || 0;
    setDelay(Math.max(0, Math.min(1000, value)));
  };

  const handleRunFromCursor = () => {
    const mainEditor = editorManager.getEditor('main');
    if (mainEditor) {
      const state = mainEditor.getState();
      const cursorPosition = state.selection.focus;
      interpreterStore.runFromPosition(cursorPosition);
    }
  };

  const handleStepToCursor = () => {
    const mainEditor = editorManager.getEditor('main');
    if (mainEditor) {
      const state = mainEditor.getState();
      const cursorPosition = state.selection.focus;
      interpreterStore.stepToPosition(cursorPosition);
    }
  };

  const handleRunWithRustWasm = async () => {
    try {
      // Get code from main editor
      const mainEditor = editorManager.getEditor('main');
      if (!mainEditor) {
        console.error('Main editor not found');
        return;
      }

      const code = mainEditor.getText();

      // Reset interpreter to clear output
      interpreterStore.reset();
      outputStore.setCollapsed(false);

      let accumulatedOutput = '';

      // Get current settings
      const tapeSize = interpreterStore.tapeSize.getValue();
      const cellSize = interpreterStore.cellSize.getValue();
      const settings = settingsStore.settings.getValue();

      // Map cell size from the store format (256, 65536, 4294967296) to bits (8, 16, 32)
      let cellBits: 8 | 16 | 32 = 8;
      if (cellSize === 256) cellBits = 8;
      else if (cellSize === 65536) cellBits = 16;
      else if (cellSize === 4294967296) cellBits = 32;

      // Run with real-time output callback
      const result = await rustWasmInterpreter.runProgram(
        code,
        '', // Input will be provided interactively
        {
          tapeSize: tapeSize,
          cellSize: cellBits,
          wrap: settings.interpreter?.wrapCells ?? true,
          wrapTape: settings.interpreter?.wrapTape ?? true,
          optimize: true,
        },
        (char, charCode) => {
          // Accumulate output and update state
          accumulatedOutput += char;
          const currentState = interpreterStore.state.getValue();
          interpreterStore.state.next({
            ...currentState,
            output: accumulatedOutput,
          });
        },
      );

      // Final update with complete output
      const finalState = interpreterStore.state.getValue();
      interpreterStore.state.next({
        ...finalState,
        output: result.output,
        isStopped: true,
      });
    } catch (error) {
      console.error('Failed to run with Rust WASM:', error);
      const currentState = interpreterStore.state.getValue();
      interpreterStore.state.next({
        ...currentState,
        output: currentState.output + `\n[Error: ${error.message}]`,
      });
    }
  };

  const handleStopRustWasm = () => {
    rustWasmInterpreter.stop();
  };

  return (
    <div className="h-10 min-h-10 border-t border-zinc-800 bg-zinc-900 text-zinc-400">
      <div className="flex items-center px-2 h-full gap-1">
        {/* Rust WASM Run button */}
        <div className="flex items-center gap-1 pr-2 border-r border-zinc-700">
          {wasmRunning ? (
            <IconButton
              icon={StopIcon}
              label="Stop Rust WASM Interpreter"
              onClick={handleStopRustWasm}
              variant="danger"
            />
          ) : (
            <IconButton
              icon={RocketLaunchIcon}
              label="Run with Rust WASM (Optimized, Fastest, No Input, No Debug)"
              onClick={handleRunWithRustWasm}
              disabled={wasmStatus !== 'ready' || isRunning}
              variant="success"
            />
          )}
        </div>

        {/* Run modes group */}
        <div className="flex items-center gap-1 pr-2 border-r border-zinc-700">
          {isPaused ? (
            <>
              {lastExecutionMode === 'turbo' && (
                <IconButton
                  icon={BoltIcon}
                  label="Continue Turbo"
                  onClick={() => interpreterStore.resumeTurbo()}
                  variant="success"
                />
              )}
              <IconButton
                icon={PlayIcon}
                label="Resume"
                onClick={() => interpreterStore.resume()}
                variant="success"
              />
            </>
          ) : (
            <>
              <IconButton
                icon={BoltIcon}
                label="Run Really Fast (No delay, rare UI updates, output may be clobbered)"
                onClick={() => interpreterStore.runTurbo()}
                disabled={isRunning}
                variant="success"
              />

              <IconButton
                icon={PlayIcon}
                label="Run Smoothly (UI updates, breakpoints respected, slowest)"
                onClick={() => interpreterStore.runSmooth()}
                disabled={isRunning}
                variant="success"
              />

              {/* Run with custom delay */}
              <div className="flex items-center">
                <IconButton
                  icon={ClockIcon}
                  label={`Run with ${delay}ms delay`}
                  onClick={() => {
                    interpreterStore.run(delay);
                    setShowDelayInput(false);
                  }}
                  disabled={isRunning}
                  variant="success"
                />
                {showDelayInput ? (
                  <input
                    type="number"
                    value={delay}
                    onChange={handleDelayChange}
                    onBlur={() => setShowDelayInput(false)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        interpreterStore.run(delay);
                        setShowDelayInput(false);
                      }
                      if (e.key === 'Escape') {
                        setShowDelayInput(false);
                      }
                    }}
                    className="ml-1 w-16 px-1 py-0.5 text-xs bg-zinc-800 border border-zinc-700 rounded text-zinc-300 focus:outline-none focus:border-zinc-600"
                    placeholder="ms"
                    min="0"
                    max="1000"
                    autoFocus
                  />
                ) : (
                  <button
                    onClick={() => setShowDelayInput(true)}
                    className="ml-1 px-1 py-0.5 text-xs bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded text-zinc-400"
                    disabled={isRunning}
                  >
                    {delay}ms
                  </button>
                )}
              </div>
            </>
          )}
        </div>

        {/* Control buttons */}
        {isRunning && !isPaused && (
          <IconButton
            icon={PauseIcon}
            label="Pause"
            onClick={() => interpreterStore.pause()}
            variant="warning"
          />
        )}

        <IconButton
          icon={StopIcon}
          label="Stop"
          onClick={() => interpreterStore.stop()}
          disabled={!isRunning}
          variant="danger"
        />

        <IconButton
          icon={ForwardIcon}
          label="Step"
          onClick={() => interpreterStore.step()}
          disabled={isRunning && !isPaused}
          variant="info"
        />

        <IconButton
          icon={CursorArrowRaysIcon}
          label="Run from cursor"
          onClick={handleRunFromCursor}
          disabled={isRunning}
          variant="success"
        />

        <IconButton
          icon={CursorArrowRaysIcon}
          label="Step to cursor"
          onClick={handleStepToCursor}
          disabled={isRunning}
          variant="info"
        />

        <div className="w-px h-6 bg-zinc-700 mx-1" />

        <IconButton
          icon={XMarkIcon}
          label="Clear Breakpoints"
          onClick={() => interpreterStore.clearBreakpoints()}
          variant="warning"
        />

        <IconButton
          icon={ArrowPathIcon}
          label="Reset"
          onClick={() => interpreterStore.reset()}
          variant="warning"
        />

        {/* Status indicator */}
        <div className="ml-auto flex items-center gap-2 text-xs">
          {/* Regular debugger_ui running status */}
          {isRunning && (
            <div className="flex items-center gap-1">
              <div
                className={`w-2 h-2 rounded-full ${
                  isWaitingForInput
                    ? 'bg-blue-500 animate-pulse'
                    : isPaused
                      ? 'bg-yellow-500'
                      : 'bg-green-500 animate-pulse'
                }`}
              />
              <span
                className={
                  isWaitingForInput
                    ? 'text-blue-500'
                    : isPaused
                      ? 'text-yellow-500'
                      : 'text-green-500'
                }
              >
                {isWaitingForInput
                  ? 'Waiting for input'
                  : isPaused
                    ? 'Paused'
                    : 'Running'}
              </span>
            </div>
          )}
          {/* WASM interpreter running status */}
          {wasmRunning && (
            <div className="flex items-center gap-1">
              <div
                className={`w-2 h-2 rounded-full ${
                  wasmWaitingForInput
                    ? 'bg-blue-500 animate-pulse'
                    : 'bg-purple-500 animate-pulse'
                }`}
              />
              <span
                className={
                  wasmWaitingForInput ? 'text-blue-500' : 'text-purple-500'
                }
              >
                {wasmWaitingForInput
                  ? 'Waiting for input (WASM)'
                  : 'Running (WASM)'}
              </span>
            </div>
          )}
          {/* Finished status */}
          {!isRunning && !wasmRunning && isStopped && (
            <div className="flex items-center gap-1">
              <div className="w-2 h-2 rounded-full bg-zinc-500" />
              <span className="text-zinc-500">Finished</span>
              {lastExecutionTime !== undefined &&
                lastOperationCount !== undefined && (
                  <span className="text-zinc-600 ml-2">
                    ({lastExecutionTime.toFixed(2)}s,{' '}
                    {Math.round(
                      lastOperationCount / lastExecutionTime,
                    ).toLocaleString()}{' '}
                    ops/s)
                  </span>
                )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
