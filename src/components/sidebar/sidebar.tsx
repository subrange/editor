import {useLocalStorageState} from "../../hooks/use-local-storage-state.tsx";
import clsx from "clsx";
import {CogIcon} from "@heroicons/react/24/outline";
import {useState} from "react";
import {interpreterStore} from "../debugger/interpreter.store.ts";
import {settingsStore} from "../../stores/settings.store.ts";
import {useStoreSubscribe} from "../../hooks/use-store-subscribe.tsx";

function SidebarTabButton({
                              icon: Icon,
                              label,
                              active,
                              onClick,
                          }: {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    active: boolean;
    onClick: () => void;
}) {
    return (
        <button
            className={clsx(
                "flex items-center justify-center w-full p-3 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 transition-all duration-200",
                {
                    "bg-zinc-800 text-zinc-200": active,
                    "hover:bg-zinc-800/50": !active
                }
            )}
            onClick={onClick}
            title={label}
        >
            <Icon className="h-8 w-8" />
        </button>
    );
}

function SettingSection({ title, children }: { title: string; children: React.ReactNode }) {
    return (
        <div className="space-y-3">
            <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">{title}</h3>
            {children}
        </div>
    );
}

function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function Sidebar() {
    const [activeTab, setActiveTab] = useLocalStorageState<'settings' | null>("sidebarTab", null);
    const [tapeSize, setTapeSize] = useState(30000);
    const [cellSize, setCellSize] = useState(256);
    const [laneCount, setLaneCount] = useLocalStorageState<number>("brainfuck-ide-lane-count", 1);
    const settings = useStoreSubscribe(settingsStore.settings);

    const handleTapeSizeChange = (value: string) => {
        const size = parseInt(value) || 30000;
        setTapeSize(Math.max(100, Math.min(150000, size)));
        interpreterStore.setTapeSize(Math.max(100, Math.min(150000, size)));
    };

    const changeCellSize = (size: number) => {
        setCellSize(size);
        interpreterStore.setCellSize(size);
    };

    return (
        <div className={clsx(
            "flex h-screen bg-zinc-900 transition-all duration-300 ease-in-out",
            {
                "w-80 min-w-80": activeTab,
                "w-12 min-w-12": !activeTab,
            }
        )}>
            {/* Sidebar buttons */}
            <div className="w-12 flex flex-col bg-zinc-900">
                <SidebarTabButton
                    icon={CogIcon}
                    label="Settings"
                    active={activeTab === 'settings'}
                    onClick={() => setActiveTab(activeTab === 'settings' ? null : 'settings')}
                />
            </div>

            {/* Content panel */}
            <div className={clsx(
                "flex-1 overflow-hidden transition-opacity duration-300",
                {
                    "opacity-0 pointer-events-none": !activeTab,
                    "opacity-100": activeTab,
                }
            )}>
                {activeTab === 'settings' && (
                    <div className="h-full overflow-y-auto w-[268px] border-l border-zinc-800">
                        {/* Header */}
                        <div className="sticky top-0 bg-zinc-900 border-b border-zinc-800 px-6 py-4 z-10">
                            <h2 className="text-lg font-semibold text-zinc-100 whitespace-nowrap">Settings</h2>
                        </div>

                        {/* Settings content */}
                        <div className="p-6 space-y-8">
                            {/* Interpreter Settings */}
                            <SettingSection title="Interpreter">
                                {/* Tape Size */}
                                <div className="space-y-2">
                                    <div className="flex items-center justify-between">
                                        <label className="text-sm font-medium text-zinc-300 whitespace-nowrap">
                                            Tape Size
                                        </label>
                                        <span className="text-xs text-zinc-500 whitespace-nowrap">
                                            {formatBytes(tapeSize)}
                                        </span>
                                    </div>
                                    <div className="relative">
                                        <input
                                            type="range"
                                            min="100"
                                            max="150000"
                                            step="100"
                                            value={tapeSize}
                                            onChange={(e) => handleTapeSizeChange(e.target.value)}
                                            className="w-full h-2 bg-zinc-700 rounded appearance-none cursor-pointer slider"
                                        />
                                        <input
                                            type="number"
                                            value={tapeSize}
                                            onChange={(e) => handleTapeSizeChange(e.target.value)}
                                            className="mt-2 w-full px-3 py-2 bg-zinc-800 text-zinc-200 text-sm rounded border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                                            placeholder="Tape size in bytes"
                                        />
                                    </div>
                                </div>

                                {/* Cell Size */}
                                <div className="flex flex-col space-y-2 mt-6">
                                    <label className="text-sm font-medium text-zinc-300 whitespace-nowrap">
                                        Cell Size
                                    </label>
                                    <div className="grid grid-cols-3 gap-2">
                                        {[
                                            { value: "256", label: "8-bit", desc: "0-255" },
                                            { value: "65536", label: "16-bit", desc: "0-65,5K" },
                                            { value: "4294967296", label: "32-bit", desc: "0-4.3B" }
                                        ].map((option) => (
                                            <button
                                                key={option.value}
                                                onClick={() => changeCellSize(parseInt(option.value, 10))}
                                                className={clsx(
                                                    "p-3 rounded border transition-all text-center",
                                                    cellSize === parseInt(option.value)
                                                        ? "bg-blue-500/20 border-blue-500 text-blue-400"
                                                        : "bg-zinc-800 border-zinc-700 text-zinc-300 hover:bg-zinc-700 hover:border-zinc-600"
                                                )}
                                            >
                                                <div className="font-medium text-sm">{option.label}</div>
                                                <div className="text-[10px] text-zinc-500 mt-1">{option.desc}</div>
                                            </button>
                                        ))}
                                    </div>
                                </div>

                                {/* Lane Count */}
                                <div className="space-y-2 mt-6">
                                    <label className="text-sm font-medium text-zinc-300">
                                        Lane Count
                                    </label>
                                    <div className="flex items-center gap-2">
                                        <input
                                            type="range"
                                            min="1"
                                            max="10"
                                            value={laneCount}
                                            onChange={e => {
                                                const value = Number(e.target.value);
                                                setLaneCount(value);
                                                interpreterStore.setLaneCount(value);
                                            }}
                                            className="flex-1 h-2 bg-zinc-800 rounded-lg appearance-none cursor-pointer
                                                     [&::-webkit-slider-thumb]:appearance-none
                                                     [&::-webkit-slider-thumb]:w-4
                                                     [&::-webkit-slider-thumb]:h-4
                                                     [&::-webkit-slider-thumb]:rounded-full
                                                     [&::-webkit-slider-thumb]:bg-zinc-400
                                                     [&::-webkit-slider-thumb]:cursor-pointer
                                                     [&::-webkit-slider-thumb]:transition-colors
                                                     [&::-webkit-slider-thumb]:hover:bg-zinc-300"
                                        />
                                        <input
                                            type="number"
                                            min="1"
                                            max="10"
                                            value={laneCount}
                                            onChange={e => {
                                                const value = Number(e.target.value);
                                                if (value >= 1 && value <= 10) {
                                                    setLaneCount(value);
                                                    interpreterStore.setLaneCount(value);
                                                }
                                            }}
                                            className="w-16 px-2 py-1 text-sm bg-zinc-800 border border-zinc-700 rounded
                                                     text-zinc-300 text-center focus:outline-none focus:border-zinc-600"
                                        />
                                    </div>
                                    <p className="text-xs text-zinc-500">
                                        Visualize tape as interleaved lanes ({laneCount} {laneCount === 1 ? 'lane' : 'lanes'})
                                    </p>
                                </div>
                            </SettingSection>

                            {/* Debugger Settings */}
                            <SettingSection title="Debugger">
                                <div className="space-y-4">
                                    <label className="flex items-center justify-between cursor-pointer group">
                                        <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                                            Compact View
                                        </span>
                                        <input
                                            type="checkbox"
                                            checked={settings?.debugger.compactView ?? false}
                                            onChange={(e) => settingsStore.setDebuggerCompactView(e.target.checked)}
                                            className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
                                        />
                                    </label>
                                    <p className="text-xs text-zinc-500 -mt-2">
                                        Show memory cells in a condensed format
                                    </p>
                                </div>
                            </SettingSection>

                            {/* Macro Settings */}
                            <SettingSection title="Macro Expansion">
                                <div className="space-y-4">
                                    <label className="flex items-center justify-between cursor-pointer group">
                                        <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                                            Strip Comments
                                        </span>
                                        <input
                                            type="checkbox"
                                            checked={settings?.macro.stripComments ?? true}
                                            onChange={(e) => settingsStore.setMacroStripComments(e.target.checked)}
                                            className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
                                        />
                                    </label>
                                    <p className="text-xs text-zinc-500 -mt-2">
                                        Remove all non-Brainfuck characters from expanded code
                                    </p>

                                    <label className="flex items-center justify-between cursor-pointer group">
                                        <span className="text-sm font-medium text-zinc-300 group-hover:text-zinc-200">
                                            Collapse Empty Lines
                                        </span>
                                        <input
                                            type="checkbox"
                                            checked={settings?.macro.collapseEmptyLines ?? true}
                                            onChange={(e) => settingsStore.setMacroCollapseEmptyLines(e.target.checked)}
                                            className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
                                        />
                                    </label>
                                    <p className="text-xs text-zinc-500 -mt-2">
                                        Remove lines that contain no Brainfuck commands
                                    </p>
                                </div>
                            </SettingSection>

                            {/* Editor Settings */}
                            {/*<SettingSection title="Editor">*/}
                            {/*    <div className="space-y-4">*/}
                            {/*        <label className="flex items-center justify-between cursor-pointer">*/}
                            {/*            <span className="text-sm font-medium text-zinc-300">Syntax highlighting</span>*/}
                            {/*            <input*/}
                            {/*                type="checkbox"*/}
                            {/*                defaultChecked*/}
                            {/*                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2"*/}
                            {/*            />*/}
                            {/*        </label>*/}

                            {/*        <label className="flex items-center justify-between cursor-pointer">*/}
                            {/*            <span className="text-sm font-medium text-zinc-300">Bracket matching</span>*/}
                            {/*            <input*/}
                            {/*                type="checkbox"*/}
                            {/*                defaultChecked*/}
                            {/*                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2"*/}
                            {/*            />*/}
                            {/*        </label>*/}
                            {/*    </div>*/}
                            {/*</SettingSection>*/}

                            {/* Debug Settings */}
                            {/*<SettingSection title="Debugger">*/}
                            {/*    <div className="space-y-4">*/}
                            {/*        <label className="flex items-center justify-between cursor-pointer">*/}
                            {/*            <span className="text-sm font-medium text-zinc-300">Show execution marker</span>*/}
                            {/*            <input*/}
                            {/*                type="checkbox"*/}
                            {/*                defaultChecked*/}
                            {/*                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2"*/}
                            {/*            />*/}
                            {/*        </label>*/}

                            {/*        <label className="flex items-center justify-between cursor-pointer">*/}
                            {/*            <span className="text-sm font-medium text-zinc-300">Auto-scroll to pointer</span>*/}
                            {/*            <input*/}
                            {/*                type="checkbox"*/}
                            {/*                defaultChecked*/}
                            {/*                className="w-4 h-4 text-blue-500 bg-zinc-800 border-zinc-600 rounded focus:ring-blue-500 focus:ring-2"*/}
                            {/*            />*/}
                            {/*        </label>*/}
                            {/*    </div>*/}
                            {/*</SettingSection>*/}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}