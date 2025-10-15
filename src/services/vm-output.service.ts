import { interpreterStore } from '../components/debugger/interpreter-facade.store';
import { vmTerminalStore } from '../stores/vm-terminal.store';

class VMOutputService {
  private isInitialized = false;
  private unsubscribe: (() => void) | null = null;

  initialize() {
    if (this.isInitialized) {
      return;
    }

    this.isInitialized = true;
    this.setupCallback();
    this.subscribeToConfigChanges();
  }

  private setupCallback() {
    const callback = (
      tape: Uint8Array | Uint16Array | Uint32Array,
      pointer: number,
    ) => {
      const config = vmTerminalStore.state.getValue().config;
      const { outCellIndex, outFlagCellIndex, clearOnRead, enabled } = config;

      if (!enabled) {
        return;
      }

      // Check if indices are valid
      if (tape.length <= Math.max(outCellIndex, outFlagCellIndex)) {
        return;
      }

      // Check if flag is set
      const flagValue = tape[outFlagCellIndex];
      if (flagValue === 1) {
        const charCode = tape[outCellIndex];
        const char = String.fromCharCode(charCode);
        vmTerminalStore.appendOutput(char);

        // Clear the flag if clearOnRead is enabled
        if (clearOnRead) {
          tape[outFlagCellIndex] = 0;
        }
      }
    };

    // Register the callback
    interpreterStore.setVMOutputCallback(callback);
  }

  private subscribeToConfigChanges() {
    // Subscribe to VM terminal config changes
    this.unsubscribe = vmTerminalStore.state.subscribe((state) => {
      const { outCellIndex, outFlagCellIndex, clearOnRead, enabled } =
        state.config;

      // Update the VM output config in the interpreter
      interpreterStore.setVMOutputConfig({
        outCellIndex,
        outFlagCellIndex,
        clearOnRead,
        sparseCellPattern: {
          start: Math.min(outCellIndex, outFlagCellIndex),
          step: 1,
          count: Math.abs(outCellIndex - outFlagCellIndex) + 1,
        },
      });
    });
  }

  destroy() {
    if (this.unsubscribe) {
      this.unsubscribe();
      this.unsubscribe = null;
    }
    interpreterStore.setVMOutputCallback(null);
    this.isInitialized = false;
  }
}

// Export singleton instance
export const vmOutputService = new VMOutputService();
