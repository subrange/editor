import { BehaviorSubject } from 'rxjs';
import { CPU_HEADER, CPU_FOOTER } from '../cpu/template';
import { formatMacro } from './ripple-assembler';
import { type Instruction } from './ripple-assembler/types';
import { editorManager } from './editor-manager.service';
import { ProgressiveMacroTokenizer } from '../components/editor/services/macro-tokenizer-progressive';

interface AssemblyToMacroState {
  isProcessing: boolean;
  lastError?: string;
  lastProcessedAt?: number;
}

class AssemblyToMacroService {
  private state$ = new BehaviorSubject<AssemblyToMacroState>({
    isProcessing: false,
  });

  public readonly state = this.state$.asObservable();

  /**
   * Process assembly output and update macro editor
   */
  async processAssemblyOutput(
    instructions: Instruction[],
    memoryData: number[],
  ): Promise<void> {
    this.state$.next({ ...this.state$.value, isProcessing: true });

    try {
      // Format the assembly instructions as macro code
      const macroContent = formatMacro(instructions, memoryData);

      // Wrap with CPU template
      const fullContent = CPU_HEADER + macroContent + CPU_FOOTER;

      // Get existing macro editor (it's created at app level)
      const macroEditor = editorManager.getEditor('macro');
      if (!macroEditor) {
        throw new Error(
          'Macro editor not found. It should be created at app initialization.',
        );
      }

      // Update macro editor content
      macroEditor.setContent(fullContent);

      // Trigger tokenization and expansion of the new content
      // This is needed because the editor component's tokenization happens
      // in a React effect which may not run immediately
      const tokenizer = macroEditor.getTokenizer();
      if (tokenizer instanceof ProgressiveMacroTokenizer) {
        const lines = fullContent.split('\n');
        tokenizer.tokenizeAllLines(lines);
      }

      this.state$.next({
        ...this.state$.value,
        isProcessing: false,
        lastProcessedAt: Date.now(),
      });
    } catch (error) {
      console.error('Error processing assembly output:', error);
      this.state$.next({
        ...this.state$.value,
        isProcessing: false,
        lastError: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  }

  /**
   * Clear any errors
   */
  clearError(): void {
    this.state$.next({ ...this.state$.value, lastError: undefined });
  }
}

export const assemblyToMacroService = new AssemblyToMacroService();
