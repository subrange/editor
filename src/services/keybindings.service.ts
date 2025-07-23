import type {Brand} from "../types/helper-types.types.ts";
import {Subject} from "rxjs";

export type AppCommand = Brand<string, "AppCommand">;
export type KeybindingState = Brand<string, "KeyBindingState">;

type KeyBinding = {
    keyState: {
        ctrl?: boolean;
        alt?: boolean;
        shift?: boolean;
        meta?: boolean;
    },
    key: string;
    command: AppCommand
}

class KeybindingsService {
    private keybindingsMap: Map<KeybindingState, Array<KeyBinding>> = new Map();
    private keybindingsOrder: Array<KeybindingState> = [];

    public signal = new Subject<AppCommand>;

    public keypressSignal = new Subject<KeyboardEvent>();

    public pushKeybindings(state: KeybindingState, keybindings: Array<KeyBinding>) {
        this.keybindingsMap.set(state, keybindings);
        this.keybindingsOrder.push(state);
    }

    public removeKeybindings(state: KeybindingState) {
        this.keybindingsMap.delete(state);
        this.keybindingsOrder = this.keybindingsOrder.filter(s => s !== state);
    }

    public handleKeyEvent(event: KeyboardEvent) {
        event.preventDefault();

        const key = event.key.toLowerCase();
        const keyState: KeyBinding["keyState"] = {
            ctrl: event.ctrlKey,
            alt: event.altKey,
            shift: event.shiftKey,
            meta: event.metaKey
        };

        console.log(`Key event: ${key} with state:`, keyState);

        let executed = false;

        for (const bindingsState of [...this.keybindingsOrder].reverse()) {
            const bindings = this.keybindingsMap.get(bindingsState);
            if (!bindings) {
                continue;
            }

            for (const binding of bindings) {
                if (
                    binding.key === key &&
                    Object.entries(binding.keyState).every(([key, value]) => value === keyState[key as keyof KeyBinding["keyState"]])
                ) {
                    this.signal.next(binding.command);

                    console.log(`Command executed: ${binding.command} with key: ${key} in state: ${bindingsState}`);
                    executed = true;

                    return;
                }
            }
        }

        // If not executed, propagate the "keydown" event
        if (!executed) {
            this.keypressSignal.next(event);
        }
    }

    public createKeybinding(keySequence: string, command: AppCommand): KeyBinding {
        const keys = keySequence.split("+").map(k => k.trim().toLowerCase());
        const keyState: KeyBinding["keyState"] = {
            ctrl: keys.includes("ctrl"),
            alt: keys.includes("alt"),
            shift: keys.includes("shift"),
            meta: keys.includes("meta")
        };
        const key = keys.find(k => !["ctrl", "alt", "shift", "meta"].includes(k)) || "";

        return { keyState, key, command };
    }
}

export const keybindingsService = new KeybindingsService();
