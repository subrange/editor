import React, { useEffect, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { XMarkIcon } from '@heroicons/react/24/outline';
// import { BrainfuckCodeBlock } from './brainfuck-code-block';
import { AssemblyCodeBlock } from './assembly-code-block';

interface MarkdownViewerProps {
  filePath: string;
  onClose: () => void;
}

export function MarkdownViewer({ filePath, onClose }: MarkdownViewerProps) {
  const [content, setContent] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [title, setTitle] = useState<string>('Documentation');

  useEffect(() => {
    fetch(filePath)
      .then((response) => {
        if (!response.ok) {
          throw new Error('Failed to load markdown file');
        }
        return response.text();
      })
      .then((text) => {
        setContent(text);

        // Extract title from the first heading
        const titleMatch = text.match(/^#\s+(.+)$/m);
        if (titleMatch) {
          setTitle(titleMatch[1]);
        } else {
          // Fallback based on filename
          if (filePath.includes('BRAINFUCK')) {
            setTitle('Brainfuck Macro Language Tutorial');
          } else if (
            filePath.includes('RIPPLE') ||
            filePath.includes('ASSEMBLY')
          ) {
            setTitle('Ripple VM Assembly & Architecture');
          } else {
            setTitle('Documentation');
          }
        }

        setLoading(false);
      })
      .catch((err) => {
        setError(err.message);
        setLoading(false);
      });
  }, [filePath]);

  return (
    <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-zinc-900 rounded-lg shadow-2xl w-full max-w-5xl h-[90vh] flex flex-col border border-zinc-700">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-700">
          <h2 className="text-xl font-semibold text-zinc-100">{title}</h2>
          <button
            onClick={onClose}
            className="p-1 hover:bg-zinc-800 rounded transition-colors"
            aria-label="Close"
          >
            <XMarkIcon className="w-5 h-5 text-zinc-400 hover:text-zinc-200" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4">
          {loading && (
            <div className="flex items-center justify-center h-full">
              <div className="text-zinc-400">Loading...</div>
            </div>
          )}

          {error && (
            <div className="flex items-center justify-center h-full">
              <div className="text-red-400">Error: {error}</div>
            </div>
          )}

          {!loading && !error && (
            <div className="prose prose-invert prose-zinc max-w-none">
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={{
                  // Custom component overrides
                  h1: ({ children }) => {
                    const id = String(children)
                      .toLowerCase()
                      .replace(/\s+/g, '-');
                    return (
                      <h1
                        id={id}
                        className="text-3xl font-bold text-zinc-100 mb-6 pb-3 border-b border-zinc-700"
                      >
                        {children}
                      </h1>
                    );
                  },
                  h2: ({ children }) => {
                    const id = String(children)
                      .toLowerCase()
                      .replace(/\s+/g, '-');
                    return (
                      <h2
                        id={id}
                        className="text-2xl font-semibold text-zinc-100 mt-8 mb-4"
                      >
                        {children}
                      </h2>
                    );
                  },
                  h3: ({ children }) => {
                    const id = String(children)
                      .toLowerCase()
                      .replace(/\s+/g, '-');
                    return (
                      <h3
                        id={id}
                        className="text-xl font-semibold text-zinc-200 mt-6 mb-3"
                      >
                        {children}
                      </h3>
                    );
                  },
                  p: ({ children }) => (
                    <p className="text-zinc-300 leading-relaxed mb-4">
                      {children}
                    </p>
                  ),
                  ul: ({ children }) => (
                    <ul className="list-disc list-inside space-y-2 mb-4 text-zinc-300">
                      {children}
                    </ul>
                  ),
                  ol: ({ children }) => (
                    <ol className="list-decimal list-inside space-y-2 mb-4 text-zinc-300">
                      {children}
                    </ol>
                  ),
                  li: ({ children }) => (
                    <li className="text-zinc-300">{children}</li>
                  ),
                  pre: ({ children }) => {
                    // Extract code content from nested structure
                    if (React.isValidElement(children) && children.props) {
                      const codeElement = children;
                      const className = codeElement.props.className || '';
                      const lang = className.replace(/^language-/, '');
                      const codeContent = String(
                        codeElement.props.children || '',
                      );

                      // Check if this is an Assembly code block
                      if (
                        lang === 'assembly' ||
                        lang === 'asm' ||
                        // Auto-detect assembly code by common patterns
                        (!lang &&
                          /^\s*(;|\/\/)|^\s*\.(data|code)|^\s*(NOP|ADD|SUB|LOAD|STORE|JAL|BEQ|LI|MOVE)\b/im.test(
                            codeContent,
                          ))
                      ) {
                        return (
                          <AssemblyCodeBlock
                            code={codeContent}
                            className="mb-4"
                          />
                        );
                      }

                      // Check if this is a Brainfuck code block
                      if (
                        lang === 'brainfuck' ||
                        lang === 'bf' ||
                        // Auto-detect brainfuck code by common patterns
                        (!lang &&
                          /^[#@{]|#define|@\w+|\{(repeat|if|for|reverse|preserve)/.test(
                            codeContent,
                          ))
                      ) {
                        return (
                          <BrainfuckCodeBlock
                            code={codeContent}
                            className="mb-4"
                          />
                        );
                      }
                    }

                    // Default pre styling
                    return (
                      <pre className="bg-zinc-800 border border-zinc-700 rounded-lg p-4 overflow-x-auto mb-4 text-zinc-300">
                        {children}
                      </pre>
                    );
                  },
                  code: ({ inline, children, className }) => {
                    if (inline) {
                      return (
                        <code className="px-1.5 py-0.5 bg-zinc-800 text-blue-300 rounded text-sm font-mono">
                          {children}
                        </code>
                      );
                    }

                    // For non-inline code, just return the content
                    // The pre component will handle the actual rendering
                    return <code className={className}>{children}</code>;
                  },
                  blockquote: ({ children }) => (
                    <blockquote className="border-l-4 border-blue-500 pl-4 my-4 text-zinc-400 italic">
                      {children}
                    </blockquote>
                  ),
                  a: ({ href, children }) => {
                    // Check if this is an internal anchor link
                    const isAnchorLink = href?.startsWith('#');

                    if (isAnchorLink) {
                      // Handle internal navigation
                      const handleClick = (e: React.MouseEvent) => {
                        e.preventDefault();
                        const targetId = href.substring(1);
                        // Convert the anchor to a valid ID format (lowercase, replace spaces with hyphens)
                        const elementId = targetId
                          .toLowerCase()
                          .replace(/\s+/g, '-');
                        const element = document.getElementById(elementId);
                        if (element) {
                          element.scrollIntoView({
                            behavior: 'smooth',
                            block: 'start',
                          });
                        }
                      };

                      return (
                        <a
                          href={href}
                          onClick={handleClick}
                          className="text-blue-400 hover:text-blue-300 underline cursor-pointer"
                        >
                          {children}
                        </a>
                      );
                    }

                    // External links open in new window
                    return (
                      <a
                        href={href}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-400 hover:text-blue-300 underline"
                      >
                        {children}
                      </a>
                    );
                  },
                  table: ({ children }) => (
                    <div className="overflow-x-auto mb-4">
                      <table className="min-w-full border border-zinc-700">
                        {children}
                      </table>
                    </div>
                  ),
                  thead: ({ children }) => (
                    <thead className="bg-zinc-800">{children}</thead>
                  ),
                  tbody: ({ children }) => (
                    <tbody className="divide-y divide-zinc-700">
                      {children}
                    </tbody>
                  ),
                  th: ({ children }) => (
                    <th className="px-4 py-2 text-left text-zinc-200 font-semibold">
                      {children}
                    </th>
                  ),
                  td: ({ children }) => (
                    <td className="px-4 py-2 text-zinc-300">{children}</td>
                  ),
                  hr: () => <hr className="my-6 border-zinc-700" />,
                  strong: ({ children }) => (
                    <strong className="font-semibold text-zinc-100">
                      {children}
                    </strong>
                  ),
                  em: ({ children }) => (
                    <em className="italic text-zinc-300">{children}</em>
                  ),
                }}
              >
                {content}
              </ReactMarkdown>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
