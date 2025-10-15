import init, {
  WasmAssembler,
  WasmLinker,
} from '../../ripple-asm/pkg/ripple_asm.js';
import type { AssemblerResult, AssemblerOptions, Label } from './types.ts';

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
  private rustAssembler: WasmAssembler | null = null;
  private rustLinker: WasmLinker | null = null;
  private options: AssemblerOptions;

  constructor(options: AssemblerOptions = {}) {
    this.options = {
      caseInsensitive: options.caseInsensitive ?? true,
      startBank: options.startBank ?? 0,
      bankSize: options.bankSize ?? 16,
      maxImmediate: options.maxImmediate ?? 65535,
      memoryOffset: options.memoryOffset ?? 2,
    };
  }

  private async ensureInitialized() {
    await initAssembler();

    if (!this.rustAssembler) {
      this.rustAssembler = WasmAssembler.newWithOptions(
        this.options.caseInsensitive!,
        this.options.bankSize!,
        this.options.maxImmediate!,
        this.options.memoryOffset!,
      );
    }

    if (!this.rustLinker) {
      this.rustLinker = new WasmLinker(this.options.bankSize!);
    }
  }

  async assemble(source: string): Promise<AssemblerResult> {
    await this.ensureInitialized();

    try {
      // First, assemble to get the object file
      const objectFile = this.rustAssembler!.assemble(source);
      console.log('Object file from WASM:', objectFile);
      console.log('Unresolved refs field:', objectFile.unresolved_references);

      // Debug: Check instruction structure
      if (objectFile.instructions && objectFile.instructions.length > 0) {
        console.log('First instruction structure:', objectFile.instructions[0]);
        const firstInst = objectFile.instructions[0];
        if (firstInst.opcode === 0x0e) {
          // LI opcode
          console.log('LI instruction found:', {
            opcode: firstInst.opcode,
            word0: firstInst.word0,
            word1: firstInst.word1,
            word2: firstInst.word2,
            word3: firstInst.word3,
          });
        }
      }

      // Check if we have unresolved references that need linking
      // The unresolved_references comes as a Map from WASM
      const unresolvedRefs =
        objectFile.unresolved_references || objectFile.unresolvedReferences;
      const hasUnresolved =
        unresolvedRefs &&
        (unresolvedRefs instanceof Map
          ? unresolvedRefs.size > 0
          : Object.keys(unresolvedRefs).length > 0);

      let finalResult;
      if (hasUnresolved) {
        const numRefs =
          unresolvedRefs instanceof Map
            ? unresolvedRefs.size
            : Object.keys(unresolvedRefs).length;
        console.log(
          'Running linker to resolve',
          numRefs,
          'references:',
          unresolvedRefs,
        );
        // Run the linker to resolve references
        try {
          const linkedProgram = this.rustLinker!.link([objectFile]);
          console.log('Linking successful');
          finalResult = {
            instructions: linkedProgram.instructions || [],
            data: linkedProgram.data || [],
            entryPoint: linkedProgram.entryPoint || 0,
          };
        } catch (linkError) {
          console.warn('Linking failed, returning unlinked result:', linkError);
          // If linking fails, return the unlinked result
          finalResult = objectFile;
        }
      } else {
        console.log('No unresolved references, skipping linker');
        // No unresolved references, use the object file as-is
        finalResult = objectFile;
      }

      // Convert labels from object to Map
      const labels = new Map<string, Label>();
      if (objectFile.labels) {
        for (const [key, value] of Object.entries(objectFile.labels)) {
          labels.set(key, value as Label);
        }
      }

      // Convert data labels from object to Map
      const dataLabels = new Map<string, number>();
      if (objectFile.dataLabels) {
        for (const [key, value] of Object.entries(objectFile.dataLabels)) {
          dataLabels.set(key, value as number);
        }
      }

      return {
        instructions: finalResult.instructions || [],
        labels,
        dataLabels,
        memoryData: finalResult.data || [],
        errors: [],
      };
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } catch (error: any) {
      // Parse error string into array
      const errorMessage = error.toString();
      const errors = errorMessage.split('\n').filter((e: string) => e.trim());

      return {
        instructions: [],
        labels: new Map(),
        dataLabels: new Map(),
        memoryData: [],
        errors,
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

  async getAvailableMnemonics(): Promise<string[]> {
    await this.ensureInitialized();
    return this.rustAssembler!.getAvailableMnemonics();
  }

  async getAvailableRegisters(): Promise<string[]> {
    await this.ensureInitialized();
    return this.rustAssembler!.getAvailableRegisters();
  }
}

// Create a factory that returns an async assembler
export function createAssembler(options: AssemblerOptions = {}) {
  return new RippleAssembler(options);
}

// Singleton for getting available mnemonics and registers
let cachedMnemonics: string[] | null = null;
let cachedRegisters: string[] | null = null;

export async function getAvailableMnemonics(): Promise<string[]> {
  if (!cachedMnemonics) {
    const assembler = new RippleAssembler();
    cachedMnemonics = await assembler.getAvailableMnemonics();
  }
  return cachedMnemonics;
}

export async function getAvailableRegisters(): Promise<string[]> {
  if (!cachedRegisters) {
    const assembler = new RippleAssembler();
    cachedRegisters = await assembler.getAvailableRegisters();
  }
  return cachedRegisters;
}
