import {useLocalStorageState} from "../../hooks/use-local-storage-state.tsx";
import { useState } from 'react';
import clsx from "clsx";
import {CogIcon, CameraIcon, DocumentIcon, HashtagIcon, AcademicCapIcon, InformationCircleIcon} from "@heroicons/react/24/outline";
import {Settings} from "./settings.tsx";
import {Snapshots} from "./snapshots.tsx";
import {Files} from "./files.tsx";
import {Marks} from "./marks.tsx";
import {Learning} from "./learning.tsx";
import {AboutModal} from "./about-modal.tsx";

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

export function Sidebar() {
    const [activeTab, setActiveTab] = useLocalStorageState<'settings' | 'snapshots' | 'files' | 'marks' | 'learning' | null>("sidebarTab", null);
    const [showAboutModal, setShowAboutModal] = useState(false);

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
                    icon={CameraIcon}
                    label="Snapshots"
                    active={activeTab === 'snapshots'}
                    onClick={() => setActiveTab(activeTab === 'snapshots' ? null : 'snapshots')}
                />
                <SidebarTabButton
                    icon={DocumentIcon}
                    label="Files"
                    active={activeTab === 'files'}
                    onClick={() => setActiveTab(activeTab === 'files' ? null : 'files')}
                />
                <SidebarTabButton
                    icon={HashtagIcon}
                    label="Marks"
                    active={activeTab === 'marks'}
                    onClick={() => setActiveTab(activeTab === 'marks' ? null : 'marks')}
                />
                <div className="flex-1" />
                <SidebarTabButton
                    icon={AcademicCapIcon}
                    label="Learning"
                    active={activeTab === 'learning'}
                    onClick={() => setActiveTab(activeTab === 'learning' ? null : 'learning')}
                />
                <SidebarTabButton
                    icon={InformationCircleIcon}
                    label="About"
                    active={false}
                    onClick={() => setShowAboutModal(true)}
                />
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
                {activeTab === 'settings' && <Settings />}
                {activeTab === 'snapshots' && <Snapshots />}
                {activeTab === 'files' && <Files />}
                {activeTab === 'marks' && <Marks />}
                {activeTab === 'learning' && <Learning />}
            </div>

            {/* About Modal */}
            {showAboutModal && <AboutModal onClose={() => setShowAboutModal(false)} />}
        </div>
    );
}