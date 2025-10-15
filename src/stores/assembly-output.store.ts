import { BehaviorSubject } from 'rxjs';
import type { Instruction, Label } from '../services/ripple-assembler/types';

export interface AssemblyOutputState {
  output: {
    instructions: Instruction[];
    labels: Map<string, Label>;
    dataLabels: Map<string, number>;
    memoryData: number[];
  } | null;
  error: string | null;
  isCompiling: boolean;
}

class AssemblyOutputStore {
  public state = new BehaviorSubject<AssemblyOutputState>({
    output: null,
    error: null,
    isCompiling: false,
  });

  private get state$() {
    return this.state;
  }

  get currentState() {
    return this.state.value;
  }

  setOutput(output: AssemblyOutputState['output']) {
    this.state.next({
      output,
      error: null,
      isCompiling: false,
    });
  }

  setError(error: string) {
    this.state.next({
      output: null,
      error,
      isCompiling: false,
    });
  }

  setCompiling(isCompiling: boolean) {
    this.state.next({
      ...this.state.value,
      isCompiling,
    });
  }

  clear() {
    this.state.next({
      output: null,
      error: null,
      isCompiling: false,
    });
  }
}

export const assemblyOutputStore = new AssemblyOutputStore();
