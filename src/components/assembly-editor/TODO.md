# Assembly Editor TODO

## Features to Implement

1. **Autocomplete** - Implement autocomplete functionality for assembly instructions and labels

2. **Error Highlighting** - Use the same error highlighting style as the macro editor

3. **BF Output Panel** - Make the BF Output panel the first and main panel in the assembly editor output

4. **Persist Last Opened Panel** - Remember and restore the last opened panel in the assembly editor

5. **Cmd+Click Navigation**
   - On label references: Jump to label definition
   - On label definitions: Show usages modal (similar to macro usages modal)

6. **Cmd+P Quick Navigation** - Show quick navigation with all labels and mark comments in the assembly file

7. **Real-time Error Highlighting** - Highlight errors on the fly as user types

8. **BF Macro Tokenization** - Tokenize and highlight BF Macro output, ignoring errors

## Implementation Status

- [x] Autocomplete
- [x] Error highlighting style
- [x] BF Output panel as main
- [x] Persist last opened panel
- [x] Cmd+click navigation
- [x] Cmd+P quick navigation
- [x] Real-time error highlighting
- [x] BF Macro tokenization