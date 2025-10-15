import { XMarkIcon } from '@heroicons/react/24/solid';
import faviconHuge from '../../favicon_huge.png';

export function AboutModal({ onClose }: { onClose: () => void }) {
  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      onClick={onClose}
    >
      <div
        className="bg-zinc-900 rounded-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto border border-zinc-700"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-6 py-4 flex items-center justify-between">
          <h2 className="text-xl font-semibold text-zinc-100">
            About Braintease IDE
          </h2>
          <button
            onClick={onClose}
            className="p-1 hover:bg-zinc-800 rounded transition-colors"
          >
            <XMarkIcon className="w-5 h-5 text-zinc-400" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Logo and Title */}
          <div className="flex items-center gap-4">
            <img
              src={faviconHuge}
              alt="Braintease IDE"
              className="w-20 h-20 rounded-lg"
            />
            <div>
              <h3 className="text-2xl font-bold text-zinc-100">
                Braintease IDE
              </h3>
              <p className="text-sm text-zinc-400 mt-1">
                Advanced Development Environment for Esoteric Programming
              </p>
            </div>
          </div>

          {/* Foreword */}
          <div className="space-y-3">
            <h4 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
              About This Project
            </h4>
            <div className="prose prose-invert prose-zinc max-w-none">
              <p className="text-zinc-400 text-sm leading-relaxed">
                This IDE represents a comprehensive development environment for
                Brainfuck. What started as a simple interpreter has evolved into
                a full-featured toolchain including:
              </p>
              <ul className="text-zinc-400 text-sm mt-2 space-y-1">
                <li>
                  • A sophisticated macro preprocessor system for readable BF
                  code — Brainfuck Advanced Language Layer System
                </li>
                <li>
                  • Visual debugger with tape visualization, breakpoints, and
                  stepping capabilities
                </li>
                <li>
                  • Ripple VM - a custom RISC-like virtual machine implementing
                  a Perfectly Engineered Non-standard Instruction Set made in
                  Brainfuck Advanced Language Layer System{' '}
                </li>
                <li>
                  • C compiler that targets both Brainfuck and Ripple VM binary
                </li>
                <li>• Custom code editor with advanced features</li>
              </ul>
              <p className="text-zinc-400 text-sm mt-3 leading-relaxed">
                The project demonstrates that even the most minimal languages
                can serve as foundations for complex software systems.
              </p>
              <p className="text-zinc-400 text-sm mt-3 leading-relaxed">
                A lot of code has been written by Claude Code, and not reviewed
                by me. Praise be to robot.
              </p>
            </div>
          </div>

          {/* Links */}
          <div className="space-y-3">
            <h4 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
              Project Links
            </h4>
            <div className="space-y-2">
              <a
                href="https://github.com/ahineya/braintease"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-3 p-3 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-lg transition-colors group"
              >
                <svg
                  className="w-5 h-5 text-zinc-400 group-hover:text-zinc-300"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    fillRule="evenodd"
                    d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                    clipRule="evenodd"
                  />
                </svg>
                <div>
                  <p className="text-sm font-medium text-zinc-200 group-hover:text-zinc-100">
                    Braintease GitHub Repository
                  </p>
                  <p className="text-xs text-zinc-500">
                    Source "code" and "documentation"
                  </p>
                </div>
              </a>

              <a
                href="https://github.com/ahineya"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-3 p-3 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-lg transition-colors group"
              >
                <svg
                  className="w-5 h-5 text-zinc-400 group-hover:text-zinc-300"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    fillRule="evenodd"
                    d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                    clipRule="evenodd"
                  />
                </svg>
                <div>
                  <p className="text-sm font-medium text-zinc-200 group-hover:text-zinc-100">
                    My Github Profile
                  </p>
                  <p className="text-xs text-zinc-500">
                    More projects and contributions
                  </p>
                </div>
              </a>
            </div>
          </div>

          {/* Tech Stack */}
          <div className="space-y-3">
            <h4 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
              Built With
            </h4>
            <div className="flex flex-wrap gap-2">
              <span className="px-2 py-1 bg-zinc-800 rounded text-xs text-zinc-400">
                React
              </span>
              <span className="px-2 py-1 bg-zinc-800 rounded text-xs text-zinc-400">
                TypeScript
              </span>
              <span className="px-2 py-1 bg-zinc-800 rounded text-xs text-zinc-400">
                Rust
              </span>
              <span className="px-2 py-1 bg-zinc-800 rounded text-xs text-zinc-400">
                WebAssembly
              </span>
              <span className="px-2 py-1 bg-zinc-800 rounded text-xs text-zinc-400">
                TailwindCSS
              </span>
            </div>
          </div>

          {/* Version */}
          <div className="pt-3 border-t border-zinc-800">
            <p className="text-xs text-zinc-500 text-center">Version 1.0.0</p>
          </div>
        </div>
      </div>
    </div>
  );
}
