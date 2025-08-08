import { BehaviorSubject } from 'rxjs';
import { CPU_HEADER, CPU_FOOTER } from '../cpu/template';
import { formatMacro } from './ripple-assembler';
import { type Instruction } from './ripple-assembler/types';
import { editorManager } from './editor-manager.service';

interface AssemblyToMacroState {
    isProcessing: boolean;
    lastError?: string;
    lastProcessedAt?: number;
}

class AssemblyToMacroService {
    private state$ = new BehaviorSubject<AssemblyToMacroState>({
        isProcessing: false
    });

    public readonly state = this.state$.asObservable();

    /**
     * Process assembly output and update macro editor
     */
    async processAssemblyOutput(instructions: Instruction[], memoryData: number[]): Promise<void> {
        this.state$.next({ ...this.state$.value, isProcessing: true });

        try {
            // Format the assembly instructions as macro code
            const macroContent = formatMacro(instructions, memoryData);
            
            // Wrap with CPU template
            const fullContent = CPU_HEADER + macroContent + CPU_FOOTER;

            // Get existing macro editor (it's created at app level)
            const macroEditor = editorManager.getEditor('macro');
            if (!macroEditor) {
                throw new Error('Macro editor not found. It should be created at app initialization.');
            }

            // Update macro editor content
            macroEditor.setContent(fullContent);

            // The ProgressiveMacroTokenizer will automatically trigger expansion
            // and update the main editor through its onStateChange callback
            
            this.state$.next({ 
                ...this.state$.value, 
                isProcessing: false,
                lastProcessedAt: Date.now()
            });
        } catch (error) {
            console.error('Error processing assembly output:', error);
            this.state$.next({ 
                ...this.state$.value, 
                isProcessing: false,
                lastError: error instanceof Error ? error.message : 'Unknown error'
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