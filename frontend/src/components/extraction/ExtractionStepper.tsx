import {
  FileCode,
  Search,
  Network,
  Brain,
  Sparkles,
  Database,
  Check,
} from 'lucide-react';
import { cn } from '@/lib/utils';

const EXTRACTION_PHASES = [
  {
    id: 'parsing',
    label: 'Parsing',
    icon: FileCode,
    description: 'Parsing HTML/CSS/JS with tree-sitter',
  },
  {
    id: 'detection',
    label: 'Detection',
    icon: Search,
    description: 'Detecting design system patterns',
  },
  {
    id: 'ontology',
    label: 'Ontology',
    icon: Network,
    description: 'Mapping to ontological categories',
  },
  {
    id: 'narsese',
    label: 'Narsese',
    icon: Brain,
    description: 'Generating Narsese statements',
  },
  {
    id: 'embedding',
    label: 'Embedding',
    icon: Sparkles,
    description: 'Creating vector embeddings',
  },
  {
    id: 'storing',
    label: 'Storing',
    icon: Database,
    description: 'Storing in knowledge graph',
  },
];

interface ExtractionStepperProps {
  currentPhase: string;
  progress: number;
  message?: string;
}

export function ExtractionStepper({ currentPhase, progress, message }: ExtractionStepperProps) {
  const currentIndex = EXTRACTION_PHASES.findIndex((p) => p.id === currentPhase);
  const currentPhaseData = EXTRACTION_PHASES[currentIndex];

  return (
    <div className="bg-surface rounded-lg border border-border p-4 space-y-4">
      {/* Stepper */}
      <div className="flex items-center justify-between">
        {EXTRACTION_PHASES.map((phase, index) => {
          const Icon = phase.icon;
          const isComplete = index < currentIndex;
          const isCurrent = index === currentIndex;

          return (
            <div key={phase.id} className="flex flex-col items-center flex-1">
              {/* Connector line before (except first) */}
              {index > 0 && (
                <div
                  className={cn(
                    'absolute h-0.5 w-full -translate-y-5',
                    isComplete ? 'bg-success' : 'bg-border'
                  )}
                  style={{ left: '-50%', width: '100%' }}
                />
              )}

              {/* Icon circle */}
              <div
                className={cn(
                  'relative w-10 h-10 rounded-full flex items-center justify-center border-2 transition-all duration-300',
                  isComplete && 'bg-success border-success text-white',
                  isCurrent && 'bg-primary-600 border-primary-600 text-white animate-pulse',
                  !isComplete && !isCurrent && 'border-border text-muted-foreground bg-surface'
                )}
              >
                {isComplete ? <Check className="w-5 h-5" /> : <Icon className="w-5 h-5" />}
              </div>

              {/* Label */}
              <span
                className={cn(
                  'text-xs mt-2 text-center',
                  isCurrent ? 'text-foreground font-medium' : 'text-muted-foreground'
                )}
              >
                {phase.label}
              </span>
            </div>
          );
        })}
      </div>

      {/* Progress bar */}
      <div className="h-2 bg-background rounded-full overflow-hidden">
        <div
          className="h-full bg-primary-500 transition-all duration-300"
          style={{ width: `${progress * 100}%` }}
        />
      </div>

      {/* Current phase description */}
      <div className="text-center p-3 bg-surface-elevated rounded-lg">
        <p className="text-sm text-foreground font-medium">
          {currentPhaseData?.description || message}
        </p>
        <p className="text-xs text-muted-foreground mt-1">{Math.round(progress * 100)}% complete</p>
      </div>
    </div>
  );
}
