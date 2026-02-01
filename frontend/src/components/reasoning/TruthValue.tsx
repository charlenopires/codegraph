import { cn } from '@/lib/utils';

interface TruthValueProps {
  frequency: number;
  confidence: number;
  size?: 'sm' | 'md' | 'lg';
  showLabels?: boolean;
  showTooltip?: boolean;
  className?: string;
}

/**
 * Visualizes NARS truth values (frequency, confidence) as dual progress bars.
 *
 * In NARS:
 * - Frequency (f): The proportion of positive evidence (0-1)
 * - Confidence (c): The amount of evidence relative to total possible (0-1)
 *
 * Combined they form the "evidential truth-value" <f, c>
 */
export function TruthValue({
  frequency,
  confidence,
  size = 'md',
  showLabels = true,
  showTooltip = true,
  className,
}: TruthValueProps) {
  const barHeights = {
    sm: 'h-1.5',
    md: 'h-2',
    lg: 'h-3',
  };

  const textSizes = {
    sm: 'text-xs',
    md: 'text-sm',
    lg: 'text-base',
  };

  // Color based on combined truth value
  const getFrequencyColor = (f: number) => {
    if (f >= 0.7) return 'bg-success';
    if (f >= 0.4) return 'bg-warning';
    return 'bg-error';
  };

  const getConfidenceColor = (c: number) => {
    if (c >= 0.7) return 'bg-primary-500';
    if (c >= 0.4) return 'bg-primary-400';
    return 'bg-primary-300';
  };

  return (
    <div className={cn('space-y-1', className)} title={showTooltip ? `Truth-value: <${frequency.toFixed(2)}, ${confidence.toFixed(2)}>` : undefined}>
      {/* Frequency bar */}
      <div className="flex items-center gap-2">
        {showLabels && (
          <span className={cn('w-20 text-muted-foreground', textSizes[size])}>
            Freq: {(frequency * 100).toFixed(0)}%
          </span>
        )}
        <div className={cn('flex-1 bg-surface-elevated rounded-full overflow-hidden', barHeights[size])}>
          <div
            className={cn('h-full rounded-full transition-all duration-300', getFrequencyColor(frequency))}
            style={{ width: `${frequency * 100}%` }}
          />
        </div>
      </div>

      {/* Confidence bar */}
      <div className="flex items-center gap-2">
        {showLabels && (
          <span className={cn('w-20 text-muted-foreground', textSizes[size])}>
            Conf: {(confidence * 100).toFixed(0)}%
          </span>
        )}
        <div className={cn('flex-1 bg-surface-elevated rounded-full overflow-hidden', barHeights[size])}>
          <div
            className={cn('h-full rounded-full transition-all duration-300', getConfidenceColor(confidence))}
            style={{ width: `${confidence * 100}%` }}
          />
        </div>
      </div>
    </div>
  );
}

/**
 * Compact inline truth value display: <f, c>
 */
export function TruthValueInline({
  frequency,
  confidence,
  className,
}: {
  frequency: number;
  confidence: number;
  className?: string;
}) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 px-2 py-0.5 rounded bg-surface-elevated text-xs font-mono',
        className
      )}
      title={`Frequency: ${(frequency * 100).toFixed(1)}%, Confidence: ${(confidence * 100).toFixed(1)}%`}
    >
      <span className="text-muted-foreground">&lt;</span>
      <span className="text-success">{frequency.toFixed(2)}</span>
      <span className="text-muted-foreground">,</span>
      <span className="text-primary-400">{confidence.toFixed(2)}</span>
      <span className="text-muted-foreground">&gt;</span>
    </span>
  );
}

/**
 * Combined truth value score (expectation)
 * Expectation = c * (f - 0.5) + 0.5
 */
export function TruthValueScore({
  frequency,
  confidence,
  className,
}: {
  frequency: number;
  confidence: number;
  className?: string;
}) {
  const expectation = confidence * (frequency - 0.5) + 0.5;

  const getColor = (e: number) => {
    if (e >= 0.7) return 'text-success';
    if (e >= 0.5) return 'text-warning';
    return 'text-error';
  };

  return (
    <span
      className={cn('font-bold', getColor(expectation), className)}
      title={`Expectation (truth-value score): ${(expectation * 100).toFixed(1)}%`}
    >
      {(expectation * 100).toFixed(0)}%
    </span>
  );
}
