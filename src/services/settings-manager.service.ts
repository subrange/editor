import { settingsStore } from '../stores/settings.store';
import { interpreterStore } from '../components/debugger/interpreter-facade.store';
import { outputStore } from '../stores/output.store';
import { vmTerminalStore } from '../stores/vm-terminal.store';
import { tapeLabelsStore } from '../stores/tape-labels.store';

interface AllSettings {
    version: string;
    exportDate: string;
    settings: {
        // Core settings from settings.store
        macro: {
            stripComments: boolean;
            collapseEmptyLines: boolean;
            autoExpand: boolean;
        };
        debugger: {
            compactView: boolean;
            viewMode: 'normal' | 'compact' | 'lane';
        };
        // Interpreter settings
        interpreter: {
            tapeSize: number;
            cellSize: number;
            laneCount: number;
        };
        // Output settings
        output: {
            collapsed: boolean;
            position: 'bottom' | 'right' | 'floating';
            width?: number;
            height?: number;
            maxLines?: number;
        };
        // VM Terminal settings
        vmTerminal: {
            enabled: boolean;
            screenWidth: number;
            screenHeight: number;
            clearChar: number;
            wrapLines: boolean;
        };
        // Tape labels
        tapeLabels: {
            lanes: Record<number, string>;
            columns: Record<number, string>;
            cells: Record<string, string>;
        };
        // Editor states (per editor ID)
        editorStates: Record<string, any>;
        // Files
        files: Array<{ name: string; content: string }>;
        // Snapshots
        snapshots: Array<any>;
    };
}

class SettingsManager {
    private readonly SETTINGS_VERSION = '1.0.0';

    exportAllSettings(): string {
        try {
            const allSettings: AllSettings = {
                version: this.SETTINGS_VERSION,
                exportDate: new Date().toISOString(),
                settings: {
                    // Core settings
                    macro: settingsStore.settings?.value?.macro || {
                        stripComments: true,
                        collapseEmptyLines: true,
                        autoExpand: false
                    },
                    debugger: settingsStore.settings?.value?.debugger || {
                        compactView: false,
                        viewMode: 'normal'
                    },
                    
                    // Interpreter settings
                    interpreter: {
                        tapeSize: interpreterStore.tapeSize?.value || 30000,
                        cellSize: interpreterStore.cellSize?.value || 256,
                        laneCount: interpreterStore.laneCount?.value || 1
                    },
                    
                    // Output settings
                    output: outputStore.state?.value || {
                        collapsed: false,
                        position: 'right',
                        width: 384,
                        height: 384,
                        maxLines: 10000
                    },
                    
                    // VM Terminal settings
                    vmTerminal: vmTerminalStore.config?.value || {
                        enabled: false,
                        screenWidth: 80,
                        screenHeight: 25,
                        clearChar: 0,
                        wrapLines: true
                    },
                    
                    // Tape labels
                    tapeLabels: tapeLabelsStore.labels?.value || { lanes: {}, columns: {}, cells: {} },
                    
                    // Editor states - collect all editor states from localStorage
                    editorStates: this.collectEditorStates(),
                    
                    // Files
                    files: this.getStoredFiles(),
                    
                    // Snapshots
                    snapshots: this.getStoredSnapshots()
                }
            };

            return JSON.stringify(allSettings, null, 2);
        } catch (error) {
            console.error('Failed to export settings:', error);
            throw new Error(`Failed to export settings: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }

    importAllSettings(jsonString: string): void {
        try {
            const imported: AllSettings = JSON.parse(jsonString);
            
            // Validate version compatibility
            if (!this.isVersionCompatible(imported.version)) {
                throw new Error(`Incompatible settings version: ${imported.version}. Expected ${this.SETTINGS_VERSION}`);
            }

            // Import core settings
            if (imported.settings.macro) {
                settingsStore.setMacroStripComments(imported.settings.macro.stripComments);
                settingsStore.setMacroCollapseEmptyLines(imported.settings.macro.collapseEmptyLines);
                settingsStore.setMacroAutoExpand(imported.settings.macro.autoExpand);
            }

            if (imported.settings.debugger) {
                settingsStore.setDebuggerViewMode(imported.settings.debugger.viewMode);
            }

            // Import interpreter settings
            if (imported.settings.interpreter) {
                interpreterStore.setTapeSize(imported.settings.interpreter.tapeSize);
                interpreterStore.setCellSize(imported.settings.interpreter.cellSize);
                interpreterStore.setLaneCount(imported.settings.interpreter.laneCount);
            }

            // Import output settings
            if (imported.settings.output) {
                const output = imported.settings.output;
                if (output.collapsed !== undefined) {
                    outputStore.setCollapsed(output.collapsed);
                }
                if (output.position) {
                    outputStore.setPosition(output.position);
                }
                if (output.width !== undefined) {
                    outputStore.setSize('width', output.width);
                }
                if (output.height !== undefined) {
                    outputStore.setSize('height', output.height);
                }
                if (output.maxLines !== undefined) {
                    outputStore.setMaxLines(output.maxLines);
                }
            }

            // Import VM Terminal settings
            if (imported.settings.vmTerminal) {
                vmTerminalStore.updateConfig(imported.settings.vmTerminal);
            }

            // Import tape labels
            if (imported.settings.tapeLabels) {
                // Clear existing labels first
                tapeLabelsStore.clearAllLabels();
                
                // Import lane labels
                Object.entries(imported.settings.tapeLabels.lanes).forEach(([lane, label]) => {
                    tapeLabelsStore.setLaneLabel(parseInt(lane), label);
                });
                
                // Import column labels
                Object.entries(imported.settings.tapeLabels.columns).forEach(([col, label]) => {
                    tapeLabelsStore.setColumnLabel(parseInt(col), label);
                });
                
                // Import cell labels
                Object.entries(imported.settings.tapeLabels.cells).forEach(([index, label]) => {
                    tapeLabelsStore.setCellLabel(parseInt(index), label);
                });
            }

            // Import editor states
            if (imported.settings.editorStates) {
                this.restoreEditorStates(imported.settings.editorStates);
            }

            // Import files
            if (imported.settings.files) {
                this.restoreFiles(imported.settings.files);
            }

            // Import snapshots
            if (imported.settings.snapshots) {
                this.restoreSnapshots(imported.settings.snapshots);
            }

        } catch (error) {
            console.error('Failed to import settings:', error);
            throw new Error(`Failed to import settings: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }

    private isVersionCompatible(version: string): boolean {
        // For now, just check exact match. In future, could add more sophisticated version checking
        return version === this.SETTINGS_VERSION;
    }

    private collectEditorStates(): Record<string, any> {
        const states: Record<string, any> = {};
        
        // Iterate through localStorage to find all editor states
        for (let i = 0; i < localStorage.length; i++) {
            const key = localStorage.key(i);
            if (key && key.startsWith('editorState_')) {
                const value = localStorage.getItem(key);
                if (value) {
                    try {
                        states[key] = JSON.parse(value);
                    } catch (e) {
                        console.warn(`Failed to parse editor state for ${key}`);
                    }
                }
            }
        }
        
        return states;
    }

    private restoreEditorStates(states: Record<string, any>): void {
        Object.entries(states).forEach(([key, value]) => {
            try {
                localStorage.setItem(key, JSON.stringify(value));
            } catch (e) {
                console.warn(`Failed to restore editor state for ${key}`);
            }
        });
    }

    private getStoredFiles(): Array<{ name: string; content: string }> {
        try {
            const stored = localStorage.getItem('brainfuck-ide-files');
            return stored ? JSON.parse(stored) : [];
        } catch {
            return [];
        }
    }

    private restoreFiles(files: Array<{ name: string; content: string }>): void {
        try {
            localStorage.setItem('brainfuck-ide-files', JSON.stringify(files));
        } catch (e) {
            console.warn('Failed to restore files');
        }
    }

    private getStoredSnapshots(): Array<any> {
        try {
            const stored = localStorage.getItem('tape-snapshots');
            return stored ? JSON.parse(stored) : [];
        } catch {
            return [];
        }
    }

    private restoreSnapshots(snapshots: Array<any>): void {
        try {
            localStorage.setItem('tape-snapshots', JSON.stringify(snapshots));
        } catch (e) {
            console.warn('Failed to restore snapshots');
        }
    }

    downloadSettingsAsFile(): void {
        const json = this.exportAllSettings();
        const blob = new Blob([json], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `brainfuck-ide-settings-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    async importSettingsFromFile(file: File): Promise<void> {
        const text = await file.text();
        this.importAllSettings(text);
    }
}

export const settingsManager = new SettingsManager();