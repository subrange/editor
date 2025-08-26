import React, { useMemo, useState } from 'react';
import { AssemblyTokenizer, assemblyTokenStyles, type AssemblyToken } from './editor/services/assembly-tokenizer';
import { ClipboardDocumentIcon, CheckIcon } from '@heroicons/react/24/outline';
import clsx from 'clsx';

interface AssemblyCodeBlockProps {
    code: string;
    className?: string;
}

export function AssemblyCodeBlock({ code, className }: AssemblyCodeBlockProps) {
    const [copied, setCopied] = useState(false);
    
    // Tokenize the code using the assembly tokenizer
    const tokenizedLines = useMemo(() => {
        const tokenizer = new AssemblyTokenizer();
        const lines = code.split('\n');
        return tokenizer.tokenizeAllLines(lines);
    }, [code]);

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(code);
            setCopied(true);
            // Reset after 2 seconds
            setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error('Failed to copy:', err);
        }
    };

    return (
        <div className={clsx("bg-zinc-950 border border-zinc-800 rounded-lg overflow-hidden relative group", className)}>
            {/* Copy button */}
            <button
                onClick={handleCopy}
                className={clsx(
                    "absolute top-2 right-2 p-1.5 rounded transition-all",
                    "opacity-0 group-hover:opacity-100",
                    copied 
                        ? "bg-green-600/20 text-green-400 border border-green-500/30"
                        : "bg-zinc-800/80 hover:bg-zinc-700/80 text-zinc-400 hover:text-zinc-200 border border-zinc-600/50"
                )}
                title={copied ? "Copied!" : "Copy code"}
            >
                {copied ? (
                    <CheckIcon className="w-4 h-4" />
                ) : (
                    <ClipboardDocumentIcon className="w-4 h-4" />
                )}
            </button>
            
            <div className="overflow-x-auto">
                <pre className="p-4 text-sm font-mono">
                    {tokenizedLines.map((lineTokens, lineIndex) => (
                        <div key={lineIndex} className="leading-relaxed">
                            {lineTokens.length === 0 ? (
                                // Empty line
                                <span>&nbsp;</span>
                            ) : (
                                lineTokens.map((token: AssemblyToken, tokenIndex: number) => {
                                    const style = assemblyTokenStyles[token.type];
                                    
                                    // Special handling for whitespace to preserve formatting
                                    if (token.type === 'whitespace') {
                                        return (
                                            <span key={tokenIndex}>
                                                {token.value}
                                            </span>
                                        );
                                    }
                                    
                                    // Apply appropriate styling based on token type
                                    return (
                                        <span
                                            key={tokenIndex}
                                            className={clsx(style)}
                                        >
                                            {token.value}
                                        </span>
                                    );
                                })
                            )}
                        </div>
                    ))}
                </pre>
            </div>
        </div>
    );
}