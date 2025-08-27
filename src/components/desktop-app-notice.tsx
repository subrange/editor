import { useState, useEffect } from 'react';
import { XMarkIcon } from '@heroicons/react/24/solid';
import { ComputerDesktopIcon, ArrowDownTrayIcon } from '@heroicons/react/24/outline';

export function DesktopAppNotice() {
    const [isElectron, setIsElectron] = useState(false);
    const [isDismissed, setIsDismissed] = useState(false);
    const [showAllVersions, setShowAllVersions] = useState(false);
    const [platform, setPlatform] = useState<'mac' | 'windows' | 'linux' | 'unknown'>('unknown');

    useEffect(() => {
        // Check if running in Electron
        const userAgent = navigator.userAgent.toLowerCase();
        const isInElectron = userAgent.includes('electron');
        setIsElectron(isInElectron);

        // Detect platform
        if (navigator.platform.toLowerCase().includes('mac')) {
            setPlatform('mac');
        } else if (navigator.platform.toLowerCase().includes('win')) {
            setPlatform('windows');
        } else if (navigator.platform.toLowerCase().includes('linux')) {
            setPlatform('linux');
        }

        // Check if already dismissed in localStorage (persists across sessions)
        const dismissed = localStorage.getItem('desktopAppNoticeDismissed');
        if (dismissed === 'true') {
            setIsDismissed(true);
        }
    }, []);

    const handleDismiss = () => {
        setIsDismissed(true);
        localStorage.setItem('desktopAppNoticeDismissed', 'true');
    };

    const getDownloadLink = () => {
        switch (platform) {
            case 'mac':
                return '/downloads/Braintease-IDE-mac.dmg';
            case 'windows':
                return '/downloads/Braintease-IDE-win.exe';
            case 'linux':
                return '/downloads/Braintease-IDE-linux.AppImage';
            default:
                return '#downloads';
        }
    };

    const getPlatformName = () => {
        switch (platform) {
            case 'mac':
                return 'macOS';
            case 'windows':
                return 'Windows';
            case 'linux':
                return 'Linux';
            default:
                return 'Desktop';
        }
    };

    // Don't show if in Electron or dismissed
    if (isElectron || isDismissed) {
        return null;
    }

    return (
        <>
        {/* All Versions Modal */}
        {showAllVersions && (
            <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
                <div className="bg-zinc-900 rounded-lg max-w-lg w-full border border-zinc-700">
                    {/* Header */}
                    <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-6 py-4 flex items-center justify-between">
                        <h2 className="text-xl font-semibold text-zinc-100">Download Braintease IDE</h2>
                        <button
                            onClick={() => setShowAllVersions(false)}
                            className="p-1 hover:bg-zinc-800 rounded transition-colors"
                        >
                            <XMarkIcon className="w-5 h-5 text-zinc-400" />
                        </button>
                    </div>
                    
                    {/* Content */}
                    <div className="p-6 space-y-4">
                        <p className="text-sm text-zinc-400">
                            Choose the version for your operating system:
                        </p>
                        
                        <div className="space-y-3">
                        {/* macOS */}
                        <div className="flex items-center gap-3 p-3 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-lg transition-colors group">
                            <div className="flex-1">
                                <p className="text-sm font-medium text-zinc-200 group-hover:text-zinc-100">🍎 macOS</p>
                                <p className="text-xs text-zinc-500">Universal DMG • macOS 10.12+</p>
                                </div>
                            <a
                                href="/downloads/Braintease-IDE-mac.dmg"
                                download
                                className="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-xs font-medium text-white transition-colors"
                            >
                                <ArrowDownTrayIcon className="w-3.5 h-3.5" />
                                Download
                            </a>
                        </div>
                        
                        {/* Windows */}
                        <div className="flex items-center gap-3 p-3 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-lg transition-colors group">
                            <div className="flex-1">
                                <p className="text-sm font-medium text-zinc-200 group-hover:text-zinc-100">🪟 Windows</p>
                                <p className="text-xs text-zinc-500">Installer EXE • Windows 10+</p>
                            </div>
                            <a
                                href="/downloads/Braintease-IDE-win.exe"
                                download
                                className="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-xs font-medium text-white transition-colors"
                            >
                                <ArrowDownTrayIcon className="w-3.5 h-3.5" />
                                Download
                            </a>
                        </div>
                        
                        {/* Linux */}
                        <div className="flex items-center gap-3 p-3 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-lg transition-colors group">
                            <div className="flex-1">
                                <p className="text-sm font-medium text-zinc-200 group-hover:text-zinc-100">🐧 Linux</p>
                                <p className="text-xs text-zinc-500">AppImage • Most distributions</p>
                            </div>
                            <a
                                href="/downloads/Braintease-IDE-linux.AppImage"
                                download
                                className="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-xs font-medium text-white transition-colors"
                            >
                                <ArrowDownTrayIcon className="w-3.5 h-3.5" />
                                Download
                            </a>
                        </div>
                    </div>
                    
                    {/* Footer */}
                    <div className="pt-3 border-t border-zinc-800">
                        <p className="text-xs text-zinc-500 text-center">
                            All downloads include the full IDE with Brainfuck and RippleVM support
                        </p>
                    </div>
                    </div>
                </div>
            </div>
        )}
        
        {/* Main Notice */}
        <div className="fixed bottom-4 right-4 max-w-md bg-blue-900/20 border border-blue-700/30 rounded-lg z-50 backdrop-blur-sm shadow-2xl">
            <div className="p-4">
                <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-2">
                        <ComputerDesktopIcon className="w-5 h-5 text-blue-400" />
                        <h3 className="text-sm font-semibold text-blue-100">Get the Desktop App</h3>
                    </div>
                    <button
                        onClick={handleDismiss}
                        className="p-1 hover:bg-blue-800/30 rounded transition-colors -mt-1 -mr-1"
                        aria-label="Dismiss notice"
                    >
                        <XMarkIcon className="w-4 h-4 text-blue-400" />
                    </button>
                </div>
                
                <p className="text-xs text-blue-200 mb-3">
                    Want Braintease IDE as a standalone desktop application?
                    <br />
                    Download the Electron app for your platform.
                </p>
                
                {platform !== 'unknown' && (
                    <div className="flex gap-2">
                        <a
                            href={getDownloadLink()}
                            download
                            className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-blue-600/30 hover:bg-blue-600/40 border border-blue-500/30 rounded text-xs font-medium text-blue-100 transition-colors"
                        >
                            <ArrowDownTrayIcon className="w-4 h-4" />
                            Download for {getPlatformName()}
                        </a>
                        <button
                            onClick={() => setShowAllVersions(true)}
                            className="px-3 py-2 hover:bg-blue-800/30 rounded text-xs text-blue-300 transition-colors"
                        >
                            All versions
                        </button>
                    </div>
                )}
                
                {platform === 'unknown' && (
                    <button
                        onClick={() => setShowAllVersions(true)}
                        className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-blue-600/30 hover:bg-blue-600/40 border border-blue-500/30 rounded text-xs font-medium text-blue-100 transition-colors"
                    >
                        <ArrowDownTrayIcon className="w-4 h-4" />
                        View Downloads
                    </button>
                )}
            </div>
        </div>
        </>
    );
}