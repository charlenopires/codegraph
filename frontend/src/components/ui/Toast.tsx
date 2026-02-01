import { useState } from 'react';
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-react';
import { useToastStore, type ToastType } from '@/stores/toast';
import { cn } from '@/lib/utils';

const TOAST_ICONS: Record<ToastType, React.ComponentType<{ className?: string }>> = {
  success: CheckCircle,
  error: AlertCircle,
  info: Info,
  warning: AlertTriangle,
};

const TOAST_STYLES: Record<ToastType, string> = {
  success: 'bg-success/10 border-success/30 text-success',
  error: 'bg-error/10 border-error/30 text-error',
  info: 'bg-primary-600/10 border-primary-600/30 text-primary-400',
  warning: 'bg-warning/10 border-warning/30 text-warning',
};

const ICON_STYLES: Record<ToastType, string> = {
  success: 'text-success',
  error: 'text-error',
  info: 'text-primary-400',
  warning: 'text-warning',
};

interface ToastItemProps {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
}

function ToastItem({ id, type, title, message, action }: ToastItemProps) {
  const [isExiting, setIsExiting] = useState(false);
  const { removeToast } = useToastStore();
  const Icon = TOAST_ICONS[type];

  const handleClose = () => {
    setIsExiting(true);
    setTimeout(() => removeToast(id), 200);
  };

  return (
    <div
      className={cn(
        'flex items-start gap-3 p-4 rounded-lg border shadow-lg backdrop-blur-sm transition-all duration-200',
        TOAST_STYLES[type],
        isExiting ? 'opacity-0 translate-x-4' : 'opacity-100 translate-x-0'
      )}
    >
      <Icon className={cn('w-5 h-5 flex-shrink-0 mt-0.5', ICON_STYLES[type])} />
      <div className="flex-1 min-w-0">
        <p className="font-medium text-foreground">{title}</p>
        {message && (
          <p className="text-sm text-muted-foreground mt-1">{message}</p>
        )}
        {action && (
          <button
            onClick={() => {
              action.onClick();
              handleClose();
            }}
            className="text-sm font-medium mt-2 hover:underline"
          >
            {action.label}
          </button>
        )}
      </div>
      <button
        onClick={handleClose}
        className="flex-shrink-0 p-1 hover:bg-white/10 rounded transition-colors"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}

export function ToastContainer() {
  const { toasts } = useToastStore();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
      {toasts.map((toast) => (
        <ToastItem key={toast.id} {...toast} />
      ))}
    </div>
  );
}
