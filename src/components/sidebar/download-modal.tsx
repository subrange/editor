import { XMarkIcon, ArrowDownTrayIcon } from '@heroicons/react/24/solid';
import { ComputerDesktopIcon } from '@heroicons/react/24/outline';

export function DownloadModal({ onClose }: { onClose: () => void }) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-zinc-900 rounded-lg max-w-lg w-full border border-zinc-700">
        {/* Header */}
        <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-6 py-4 flex items-center justify-between">
          <h2 className="text-xl font-semibold text-zinc-100">
            Download Braintease IDE
          </h2>
          <button
            onClick={onClose}
            className="p-1 hover:bg-zinc-800 rounded transition-colors"
          >
            <XMarkIcon className="w-5 h-5 text-zinc-400" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          <div className="flex items-center gap-3 mb-4">
            <ComputerDesktopIcon className="w-12 h-12 text-blue-400" />
            <div>
              <p className="text-sm text-zinc-300">
                Get the desktop app for a better experience. It is wrapped in
                Electron though, so expect amazing performance and stability.
              </p>
            </div>
          </div>

          <div className="space-y-3">
            {/* macOS */}
            <a
              href="/downloads/Braintease-IDE-mac.dmg"
              download
              className="flex items-center gap-3 p-3 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-lg transition-colors group cursor-pointer"
            >
              <div className="flex-1">
                <p className="text-sm font-medium text-zinc-200 group-hover:text-zinc-100">
                  🍎 macOS
                </p>
                <p className="text-xs text-zinc-500">
                  Universal DMG • macOS 10.12+
                </p>
              </div>
              <div className="flex items-center gap-2 px-3 py-1.5 bg-zinc-700/50 group-hover:bg-blue-600 rounded text-xs font-medium text-zinc-300 group-hover:text-white transition-colors">
                <ArrowDownTrayIcon className="w-3.5 h-3.5" />
                Download
              </div>
            </a>

            {/* Windows */}
            <a
              href="/downloads/Braintease-IDE-win.exe"
              download
              className="flex items-center gap-3 p-3 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-lg transition-colors group cursor-pointer"
            >
              <div className="flex-1">
                <p className="text-sm font-medium text-zinc-200 group-hover:text-zinc-100">
                  🪟 Windows
                </p>
                <p className="text-xs text-zinc-500">
                  Installer EXE • Windows 10+
                </p>
              </div>
              <div className="flex items-center gap-2 px-3 py-1.5 bg-zinc-700/50 group-hover:bg-blue-600 rounded text-xs font-medium text-zinc-300 group-hover:text-white transition-colors">
                <ArrowDownTrayIcon className="w-3.5 h-3.5" />
                Download
              </div>
            </a>

            {/* Linux */}
            <a
              href="/downloads/Braintease-IDE-linux.AppImage"
              download
              className="flex items-center gap-3 p-3 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-lg transition-colors group cursor-pointer"
            >
              <div className="flex-1">
                <p className="text-sm font-medium text-zinc-200 group-hover:text-zinc-100">
                  🐧 Linux
                </p>
                <p className="text-xs text-zinc-500">
                  AppImage • Most distributions
                </p>
              </div>
              <div className="flex items-center gap-2 px-3 py-1.5 bg-zinc-700/50 group-hover:bg-blue-600 rounded text-xs font-medium text-zinc-300 group-hover:text-white transition-colors">
                <ArrowDownTrayIcon className="w-3.5 h-3.5" />
                Download
              </div>
            </a>
          </div>

          {/* Footer */}
          <div className="pt-3 border-t border-zinc-800">
            <p className="text-xs text-zinc-500 text-center">
              All downloads include the full IDE with Brainfuck and RippleVM
              support
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
