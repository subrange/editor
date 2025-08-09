import init, { WasmAssembler as RustWasmAssembler, WasmLinker, WasmFormatter } from '../../ripple-asm/pkg/ripple_asm.js';
import type { Instruction, AssemblerResult, AssemblerOptions, Label } from './types.ts';

let wasmInitialized = false;
let initPromise: Promise<void> | null = null;

// Initialize WASM module
export async function initAssembler() {
    if (wasmInitialized) return;
    
    if (!initPromise) {
        initPromise = (async () => {
            await init();
            wasmInitialized = true;
            console.log('Rust WASM Assembler initialized');
        })();
    }
    
    await initPromise;
}

export class RippleAssembler {
    private rustAssembler: RustWasmAssembler | null = null;
    private options: AssemblerOptions;
    
    constructor(options: AssemblerOptions = {}) {
        this.options = {
            caseInsensitive: options.caseInsensitive ?? true,
            startBank: options.startBank ?? 0,
            bankSize: options.bankSize ?? 16,
            maxImmediate: options.maxImmediate ?? 65535,
            dataOffset: options.dataOffset ?? 2
        };
    }
    
    private async ensureInitialized() {
        await initAssembler();
        
        if (!this.rustAssembler) {
            this.rustAssembler = new RustWasmAssembler(
                this.options.caseInsensitive!,
                this.options.bankSize!,
                this.options.maxImmediate!,
                this.options.dataOffset!
            );
        }
    }
    
    async assemble(source: string): Promise<AssemblerResult> {
        await this.ensureInitialized();
        
        try {
            const result = this.rustAssembler!.assemble(source);
            
            // Convert labels from object to Map
            const labels = new Map<string, Label>();
            if (result.labels) {
                for (const [key, value] of Object.entries(result.labels)) {
                    labels.set(key, value as Label);
                }
            }
            
            // Convert data labels from object to Map
            const dataLabels = new Map<string, number>();
            if (result.dataLabels) {
                for (const [key, value] of Object.entries(result.dataLabels)) {
                    dataLabels.set(key, value as number);
                }
            }
            
            // Handle unresolved references
            const unresolvedRefs = result.unresolved_references || {};
            const hasUnresolved = Object.keys(unresolvedRefs).length > 0;
            
            // If there are unresolved references, we can still return the result
            // but we should note them in some way
            if (hasUnresolved) {
                console.warn('Unresolved references:', unresolvedRefs);
            }
            
            return {
                instructions: result.instructions || [],
                labels,
                dataLabels,
                memoryData: result.data || [],
                errors: []
            };
        } catch (error: any) {
            // Parse error string into array
            const errorMessage = error.toString();
            const errors = errorMessage.split('\n').filter((e: string) => e.trim());
            
            return {
                instructions: [],
                labels: new Map(),
                dataLabels: new Map(),
                memoryData: [],
                errors
            };
        }
    }
    
    async assembleToMacro(source: string): Promise<string> {
        await this.ensureInitialized();
        
        try {
            return this.rustAssembler!.assembleToMacro(source);
        } catch (error) {
            throw new Error(`Assembly failed: ${error}`);
        }
    }
    
    async assembleToBinary(source: string): Promise<Uint8Array> {
        await this.ensureInitialized();
        
        try {
            return this.rustAssembler!.assembleToBinary(source);
        } catch (error) {
            throw new Error(`Assembly failed: ${error}`);
        }
    }
}

// Create a factory that returns an async assembler
export function createAssembler(options: AssemblerOptions = {}) {
    return new RippleAssembler(options);
}