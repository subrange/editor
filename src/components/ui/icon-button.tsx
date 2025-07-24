import { Tooltip } from './tooltip';

type ToolbarButtonProps = {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    onClick: () => void;
    disabled?: boolean;
    variant?: 'default' | 'success' | 'danger' | 'warning' | 'info';
    tooltipSide?: 'top' | 'right' | 'bottom' | 'left';
    tooltipAlign?: 'start' | 'center' | 'end';
}

export function IconButton({
    icon: Icon, 
    label, 
    onClick, 
    disabled = false, 
    variant = 'default',
    tooltipSide = 'top',
    tooltipAlign = 'center'
}: ToolbarButtonProps) {
    const variantStyles = {
        default: 'text-zinc-400 hover:text-white hover:bg-zinc-800',
        success: 'text-green-500 hover:text-green-400 hover:bg-green-950',
        danger: 'text-red-500 hover:text-red-400 hover:bg-red-950',
        warning: 'text-yellow-500 hover:text-yellow-400 hover:bg-yellow-950',
        info: 'text-blue-500 hover:text-blue-400 hover:bg-blue-950'
    };

    const button = (
        <button
            className={`p-1.5 rounded transition-all ${
                disabled
                    ? 'text-zinc-600 cursor-not-allowed'
                    : variantStyles[variant as keyof typeof variantStyles]
            }`}
            onClick={onClick}
            disabled={disabled}
        >
            <Icon className="w-4 h-4"/>
        </button>
    );

    if (disabled) {
        return button;
    }

    return (
        <Tooltip content={label} side={tooltipSide} align={tooltipAlign}>
            {button}
        </Tooltip>
    );
}