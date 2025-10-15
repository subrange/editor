import type { Brand } from '../types/helper-types.types.ts';
import { Subject } from 'rxjs';

export type AppCommand = Brand<string, 'AppCommand'>;
export type KeybindingState = Brand<string, 'KeyBindingState'>;

type KeyBinding = {
  keyState: {
    ctrl?: boolean;
    alt?: boolean;
    shift?: boolean;
    meta?: boolean;
  };
  key: string;
  command: AppCommand;
};

class KeybindingsService {
  private keybindingsMap: Map<KeybindingState, Array<KeyBinding>> = new Map();
  private keybindingsOrder: Array<KeybindingState> = [];

  public signal = new Subject<AppCommand>();

  public keypressSignal = new Subject<KeyboardEvent>();

  public pushKeybindings(
    state: KeybindingState,
    keybindings: Array<KeyBinding>,
  ) {
    this.keybindingsMap.set(state, keybindings);
    this.keybindingsOrder.push(state);
  }

  public removeKeybindings(state: KeybindingState) {
    this.keybindingsMap.delete(state);
    this.keybindingsOrder = this.keybindingsOrder.filter((s) => s !== state);
  }

  public handleKeyEvent(event: KeyboardEvent) {
    // If we are in an input field, do not handle key events
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return;
    }

    // Check if interpreter is waiting for input - if so, let the event through
    // This allows the IO component to capture input for Brainfuck programs
    const interpreterStore = (window as any).interpreterStore;
    if (
      interpreterStore &&
      interpreterStore.state &&
      interpreterStore.state.getValue
    ) {
      const state = interpreterStore.state.getValue();
      if (state.isWaitingForInput) {
        return; // Let the event propagate for input handling
      }
    }

    // Also let's allow default behavior for some keys like F1-F12, Escape, cmd+R, cmd+opt+I, cmd+Q
    if (
      [
        'F1',
        'F2',
        'F3',
        'F4',
        'F5',
        'F6',
        'F7',
        'F8',
        'F9',
        'F10',
        'F11',
        'F12',
        'Escape',
      ].includes(event.key) ||
      (event.metaKey && event.key === 'r') ||
      (event.metaKey && event.altKey && event.key === 'i') ||
      (event.metaKey && event.key === 'q') ||
      (event.metaKey && event.key === 'w') || // Allow closing tabs/windows
      (event.metaKey && event.key === 'n') || // Allow new window
      (event.metaKey && event.key === 't') || // Allow new tab
      (event.metaKey && event.shiftKey && event.key === 't') || // Allow reopening closed tab
      (event.metaKey && event.key === 'h') || // Allow hiding app
      (event.metaKey && event.key === 'm')
    ) {
      // Allow minimizing
      return;
    }

    event.preventDefault();

    const key = event.key.toLowerCase();
    const keyState: KeyBinding['keyState'] = {
      ctrl: event.ctrlKey,
      alt: event.altKey,
      shift: event.shiftKey,
      meta: event.metaKey,
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
          Object.entries(binding.keyState).every(
            ([key, value]) =>
              value === keyState[key as keyof KeyBinding['keyState']],
          )
        ) {
          this.signal.next(binding.command);

          console.log(
            `Command executed: ${binding.command} with key: ${key} in state: ${bindingsState}`,
          );
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

  public createKeybinding(
    keySequence: string,
    command: AppCommand,
  ): KeyBinding {
    const keys = keySequence.split('+').map((k) => k.trim().toLowerCase());
    const keyState: KeyBinding['keyState'] = {
      ctrl: keys.includes('ctrl'),
      alt: keys.includes('alt'),
      shift: keys.includes('shift'),
      meta: keys.includes('meta'),
    };
    const key =
      keys.find((k) => !['ctrl', 'alt', 'shift', 'meta'].includes(k)) || '';

    return { keyState, key, command };
  }
}

export const keybindingsService = new KeybindingsService();
