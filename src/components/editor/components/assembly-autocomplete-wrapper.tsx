import {useState, useEffect, useRef} from 'react';
import { useStoreSubscribeToField } from '../../../hooks/use-store-subscribe.tsx';
import { EditorStore } from '../stores/editor.store.ts';
import { AssemblyAutocomplete } from './assembly-autocomplete.tsx';
import { LINE_PADDING_LEFT, LINE_PADDING_TOP, CHAR_HEIGHT } from '../constants.ts';

interface AssemblyAutocompleteWrapperProps {
    store: EditorStore;
    charWidth: number;
}

export function AssemblyAutocompleteWrapper({ store, charWidth }: AssemblyAutocompleteWrapperProps) {
    const selection = useStoreSubscribeToField(store.editorState, 'selection');
    const lines = useStoreSubscribeToField(store.editorState, 'lines');
    const [visible, setVisible] = useState(false);
    const [position, setPosition] = useState({ x: 0, y: 0 });
    const [currentWord, setCurrentWord] = useState('');
    const [labels, setLabels] = useState<string[]>([]);

    // Extract labels from the current code
    useEffect(() => {
        const extractedLabels: string[] = [];
        for (const line of lines) {
            const labelMatch = line.text.match(/^([a-zA-Z_][a-zA-Z0-9_]*):/);
            if (labelMatch) {
                extractedLabels.push(labelMatch[1]);
            }
        }
        setLabels(extractedLabels);
    }, [lines]);

    const previousLineRef = useRef<string>("");
    const previousCursorRef = useRef<{line: number, column: number}>({ line: 0, column: 0 });

    useEffect(() => {
        const cursorPos = selection.focus;
        if (cursorPos.line >= lines.length) {
            setVisible(false);
            return;
        }

        const line = lines[cursorPos.line].text;
        const previousLine = previousLineRef.current;
        const previousCursor = previousCursorRef.current;
        
        // Store current state for next comparison
        previousLineRef.current = line;
        previousCursorRef.current = cursorPos;
        
        // Check if we just typed something (line changed or cursor moved forward by 1)
        const justTyped = (
            cursorPos.line === previousCursor.line && 
            cursorPos.column === previousCursor.column + 1 &&
            line !== previousLine
        );
        
        // Only show autocomplete if we just typed
        if (!justTyped && !visible) {
            return;
        }

        const beforeCursor = line.slice(0, cursorPos.column);
        
        // Check if we're typing a word (instruction, register, or directive)
        const wordMatch = beforeCursor.match(/(\.|[A-Za-z]+)$/);
        
        if (wordMatch) {
            const word = wordMatch[0];
            // Show autocomplete for directives (starting with .) or words with 2+ characters
            if ((word.startsWith('.') || word.length >= 2) && (justTyped || visible)) {
                setCurrentWord(word);
                setVisible(true);
                
                // Calculate position
                const x = LINE_PADDING_LEFT + (cursorPos.column - word.length) * charWidth;
                const y = LINE_PADDING_TOP + cursorPos.line * CHAR_HEIGHT;
                setPosition({ x, y });
            } else {
                setVisible(false);
            }
        } else {
            setVisible(false);
        }
    }, [selection, lines, charWidth, visible]);

    const handleSelect = (completion: string) => {
        // Replace the current word with the completion
        const cursorPos = selection.focus;
        const line = lines[cursorPos.line].text;
        const beforeCursor = line.slice(0, cursorPos.column);
        const wordMatch = beforeCursor.match(/(\.|[A-Za-z]+)$/);
        
        if (wordMatch) {
            const wordStart = cursorPos.column - wordMatch[0].length;
            store.replaceRange(
                { line: cursorPos.line, column: wordStart },
                cursorPos,
                completion
            );
        }
        
        setVisible(false);
    };

    const handleDismiss = () => {
        setVisible(false);
    };

    return (
        <AssemblyAutocomplete
            visible={visible}
            x={position.x}
            y={position.y}
            currentWord={currentWord}
            labels={labels}
            onSelect={handleSelect}
            onDismiss={handleDismiss}
        />
    );
}