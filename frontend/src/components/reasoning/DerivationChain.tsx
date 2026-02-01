import { ArrowDown, Brain, Lightbulb, Target } from 'lucide-react';
import { cn } from '@/lib/utils';

interface DerivationChainProps {
  inputStatements: string[];
  derivedStatements: string[];
  conclusion?: string;
  className?: string;
}

/**
 * Visualizes the NARS inference chain from input through derivation to conclusion.
 */
export function DerivationChain({
  inputStatements,
  derivedStatements,
  conclusion,
  className,
}: DerivationChainProps) {
  return (
    <div className={cn('space-y-4', className)}>
      {/* Input statements */}
      <div className="space-y-2">
        <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
          <Brain className="w-4 h-4 text-primary-400" />
          Input (from LLM translation)
        </div>
        <div className="pl-6 space-y-1">
          {inputStatements.map((stmt, i) => (
            <div
              key={i}
              className="flex items-center gap-2 p-2 bg-primary-600/10 border border-primary-600/20 rounded"
            >
              <span className="w-5 h-5 rounded-full bg-primary-600/30 flex items-center justify-center text-xs font-mono text-primary-400">
                {i + 1}
              </span>
              <code className="text-sm font-mono text-foreground">{stmt}</code>
            </div>
          ))}
        </div>
      </div>

      {/* Arrow */}
      {derivedStatements.length > 0 && (
        <div className="flex items-center justify-center">
          <div className="flex flex-col items-center text-muted-foreground">
            <ArrowDown className="w-5 h-5" />
            <span className="text-xs">NARS Inference</span>
            <span className="text-xs">(100 cycles)</span>
          </div>
        </div>
      )}

      {/* Derived statements */}
      {derivedStatements.length > 0 && (
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
            <Lightbulb className="w-4 h-4 text-accent-400" />
            Derived (NARS reasoning)
          </div>
          <div className="pl-6 space-y-1">
            {derivedStatements.map((stmt, i) => (
              <div
                key={i}
                className="flex items-center gap-2 p-2 bg-accent-600/10 border border-accent-600/20 rounded"
              >
                <span className="w-5 h-5 rounded-full bg-accent-600/30 flex items-center justify-center text-xs font-mono text-accent-400">
                  D{i + 1}
                </span>
                <code className="text-sm font-mono text-foreground flex-1">{stmt}</code>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Conclusion */}
      {conclusion && (
        <>
          <div className="flex items-center justify-center">
            <div className="flex flex-col items-center text-muted-foreground">
              <ArrowDown className="w-5 h-5" />
              <span className="text-xs">Result</span>
            </div>
          </div>

          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
              <Target className="w-4 h-4 text-success" />
              Conclusion
            </div>
            <div className="pl-6">
              <div className="p-3 bg-success/10 border border-success/30 rounded">
                <code className="text-sm font-mono text-foreground">{conclusion}</code>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

/**
 * Compact workflow diagram showing the neural-symbolic pipeline.
 */
export function NeuralSymbolicWorkflow({ className }: { className?: string }) {
  const steps = [
    { label: 'Query', icon: '🔍', color: 'bg-primary-600/20 border-primary-600/30' },
    { label: 'LLM', icon: '🧠', color: 'bg-accent-600/20 border-accent-600/30' },
    { label: 'Narsese', icon: '📝', color: 'bg-warning/20 border-warning/30' },
    { label: 'NARS', icon: '⚡', color: 'bg-success/20 border-success/30' },
    { label: 'Results', icon: '✅', color: 'bg-primary-600/20 border-primary-600/30' },
  ];

  return (
    <div className={cn('flex items-center gap-2 overflow-x-auto py-2', className)}>
      {steps.map((step, i) => (
        <div key={step.label} className="flex items-center gap-2">
          <div
            className={cn(
              'flex flex-col items-center p-2 rounded-lg border min-w-[60px]',
              step.color
            )}
          >
            <span className="text-lg">{step.icon}</span>
            <span className="text-xs font-medium text-foreground mt-1">{step.label}</span>
          </div>
          {i < steps.length - 1 && (
            <ArrowDown className="w-4 h-4 text-muted-foreground rotate-[-90deg]" />
          )}
        </div>
      ))}
    </div>
  );
}

/**
 * Match score breakdown showing vector/graph/NARS weights.
 */
export function MatchScoreBreakdown({
  vectorScore,
  graphScore,
  narsScore,
  totalScore,
  className,
}: {
  vectorScore: number;
  graphScore: number;
  narsScore: number;
  totalScore: number;
  className?: string;
}) {
  const weights = { vector: 0.4, graph: 0.3, nars: 0.3 };

  return (
    <div className={cn('space-y-2', className)}>
      <div className="flex items-center justify-between text-sm">
        <span className="text-muted-foreground">Total Match Score</span>
        <span className="font-bold text-foreground">{(totalScore * 100).toFixed(1)}%</span>
      </div>

      <div className="space-y-1">
        <ScoreBar
          label="Vector"
          score={vectorScore}
          weight={weights.vector}
          color="bg-primary-500"
        />
        <ScoreBar
          label="Graph"
          score={graphScore}
          weight={weights.graph}
          color="bg-accent-500"
        />
        <ScoreBar
          label="NARS"
          score={narsScore}
          weight={weights.nars}
          color="bg-warning"
        />
      </div>

      <p className="text-xs text-muted-foreground">
        Weighted: 40% vector + 30% graph + 30% NARS
      </p>
    </div>
  );
}

function ScoreBar({
  label,
  score,
  weight,
  color,
}: {
  label: string;
  score: number;
  weight: number;
  color: string;
}) {
  return (
    <div className="flex items-center gap-2">
      <span className="w-12 text-xs text-muted-foreground">{label}</span>
      <div className="flex-1 h-2 bg-surface-elevated rounded-full overflow-hidden">
        <div
          className={cn('h-full rounded-full', color)}
          style={{ width: `${score * 100}%` }}
        />
      </div>
      <span className="w-16 text-xs text-right text-muted-foreground">
        {(score * 100).toFixed(0)}% × {weight * 100}%
      </span>
    </div>
  );
}
