import {
    PlayIcon,
    StopIcon,
    ArrowPathIcon,
    BoltIcon,
    ClockIcon,
    ChevronRightIcon
} from '@heroicons/react/24/solid';
import { interpreterStore } from "./interpreter.store.ts";
import { useStoreSubscribeToField } from "../../hooks/use-store-subscribe.tsx";
import { useState } from 'react';

type ToolbarButtonProps = {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    onClick: () => void;
    disabled?: boolean;
    variant?: 'default' | 'success' | 'danger' | 'warning' | 'info';
}

function IconButton({icon: Icon, label, onClick, disabled = false, variant = 'default'}: ToolbarButtonProps) {
    const variantStyles = {
        default: 'text-zinc-400 hover:text-white hover:bg-zinc-800',
        success: 'text-green-500 hover:text-green-400 hover:bg-green-950',
        danger: 'text-red-500 hover:text-red-400 hover:bg-red-950',
        warning: 'text-yellow-500 hover:text-yellow-400 hover:bg-yellow-950',
        info: 'text-blue-500 hover:text-blue-400 hover:bg-blue-950'
    };

    return (
        <button
            className={`p-1.5 rounded transition-all ${
                disabled
                    ? 'text-zinc-600 cursor-not-allowed'
                    : variantStyles[variant as keyof typeof variantStyles]
            }`}
            onClick={onClick}
            disabled={disabled}
            title={label}
        >
            <Icon className="w-4 h-4"/>
        </button>
    );
}

function Toolbar() {
    const isRunning = useStoreSubscribeToField(interpreterStore.state, 'isRunning');
    const [delay, setDelay] = useState(150);
    const [showDelayInput, setShowDelayInput] = useState(false);

    const handleDelayChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const value = parseInt(e.target.value) || 0;
        setDelay(Math.max(0, Math.min(1000, value))); // Clamp between 0-1000ms
    };

    return (
        <div className="h-10 border-t border-zinc-800 bg-zinc-900 text-zinc-400">
            <div className="flex items-center px-2 h-full gap-1">
                {/* Run modes group */}
                <div className="flex items-center gap-1 pr-2 border-r border-zinc-700">
                    <IconButton
                        icon={BoltIcon}
                        label="Run Fast"
                        onClick={() => interpreterStore.runImmediately()}
                        disabled={isRunning}
                        variant="success"
                    />

                    <IconButton
                        icon={PlayIcon}
                        label="Run Normal"
                        onClick={() => interpreterStore.runSmooth()}
                        disabled={isRunning}
                        variant="success"
                    />

                    {/* Run with custom delay */}
                    <div className="flex items-center">
                        <IconButton
                            icon={ClockIcon}
                            label={`Run with ${delay}ms delay`}
                            onClick={() => {
                                interpreterStore.run(delay);
                                setShowDelayInput(false);
                            }}
                            disabled={isRunning}
                            variant="success"
                        />
                        {showDelayInput ? (
                            <input
                                type="number"
                                value={delay}
                                onChange={handleDelayChange}
                                onBlur={() => setShowDelayInput(false)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter') {
                                        interpreterStore.run(delay);
                                        setShowDelayInput(false);
                                    }
                                    if (e.key === 'Escape') {
                                        setShowDelayInput(false);
                                    }
                                }}
                                className="ml-1 w-16 px-1 py-0.5 text-xs bg-zinc-800 border border-zinc-700 rounded text-zinc-300 focus:outline-none focus:border-zinc-600"
                                placeholder="ms"
                                min="0"
                                max="1000"
                                autoFocus
                            />
                        ) : (
                            <button
                                onClick={() => setShowDelayInput(true)}
                                className="ml-1 px-1 py-0.5 text-xs bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded text-zinc-400"
                                disabled={isRunning}
                            >
                                {delay}ms
                            </button>
                        )}
                    </div>
                </div>

                {/* Control buttons */}
                <IconButton
                    icon={StopIcon}
                    label="Stop"
                    onClick={() => interpreterStore.stop()}
                    disabled={!isRunning}
                    variant="danger"
                />

                <IconButton
                    icon={ChevronRightIcon}
                    label="Step"
                    onClick={() => interpreterStore.step()}
                    disabled={isRunning}
                    variant="info"
                />

                <div className="w-px h-6 bg-zinc-700 mx-1" />

                <IconButton
                    icon={ArrowPathIcon}
                    label="Reset"
                    onClick={() => interpreterStore.reset()}
                    variant="warning"
                />

                {/* Status indicator */}
                <div className="ml-auto flex items-center gap-2 text-xs">
                    {isRunning && (
                        <div className="flex items-center gap-1">
                            <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                            <span className="text-green-500">Running</span>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

export { Toolbar };