import { useState } from 'react';
import { Brain, ChevronDown, ChevronRight, Lightbulb, ArrowRight } from 'lucide-react';
import { TruthValueInline } from './TruthValue';
import { cn } from '@/lib/utils';

interface NarseseStatement {
  statement: string;
  frequency?: number;
  confidence?: number;
  type?: 'input' | 'derived' | 'conclusion';
}

interface NarsPanelProps {
  statements: NarseseStatement[];
  title?: string;
  showExplanation?: boolean;
  intent?: string;
  className?: string;
}

/**
 * Parses a Narsese statement string to extract the truth value if present.
 * Format: "<statement> {frequency confidence}" or "<statement> %frequency;confidence%"
 */
function parseNarsese(raw: string): NarseseStatement {
  // Try to match {f c} format
  const bracketMatch = raw.match(/^(.+?)\s*\{(\d*\.?\d+)\s+(\d*\.?\d+)\}$/);
  if (bracketMatch) {
    return {
      statement: bracketMatch[1].trim(),
      frequency: parseFloat(bracketMatch[2]),
      confidence: parseFloat(bracketMatch[3]),
    };
  }

  // Try to match %f;c% format
  const percentMatch = raw.match(/^(.+?)\s*%(\d*\.?\d+);(\d*\.?\d+)%$/);
  if (percentMatch) {
    return {
      statement: percentMatch[1].trim(),
      frequency: parseFloat(percentMatch[2]),
      confidence: parseFloat(percentMatch[3]),
    };
  }

  return { statement: raw };
}

/**
 * Converts Narsese to a more human-readable format.
 */
function humanizeNarsese(statement: string): string {
  // Handle inheritance: <A --> B>
  const inheritance = statement.match(/<(.+?)\s*-->\s*(.+?)>/);
  if (inheritance) {
    return `"${inheritance[1]}" is a kind of "${inheritance[2]}"`;
  }

  // Handle similarity: <A <-> B>
  const similarity = statement.match(/<(.+?)\s*<->\s*(.+?)>/);
  if (similarity) {
    return `"${similarity[1]}" is similar to "${similarity[2]}"`;
  }

  // Handle implication: <A ==> B>
  const implication = statement.match(/<(.+?)\s*==>\s*(.+?)>/);
  if (implication) {
    return `If "${implication[1]}" then "${implication[2]}"`;
  }

  // Handle instance: {x} --> A
  const instance = statement.match(/\{(.+?)\}\s*-->\s*(.+)/);
  if (instance) {
    return `"${instance[1]}" is an instance of "${instance[2]}"`;
  }

  // Handle property: A --> [x]
  const property = statement.match(/(.+?)\s*-->\s*\[(.+?)\]/);
  if (property) {
    return `"${property[1]}" has property "${property[2]}"`;
  }

  return statement;
}

function StatementRow({ statement, index }: { statement: NarseseStatement; index: number }) {
  const [isExpanded, setIsExpanded] = useState(false);
  const hasDetails = statement.frequency !== undefined && statement.confidence !== undefined;

  const typeColors = {
    input: 'border-l-primary-500',
    derived: 'border-l-accent-500',
    conclusion: 'border-l-success',
  };

  return (
    <div
      className={cn(
        'bg-surface-elevated rounded-lg border border-border overflow-hidden',
        statement.type && `border-l-2 ${typeColors[statement.type]}`
      )}
    >
      <button
        onClick={() => hasDetails && setIsExpanded(!isExpanded)}
        className={cn(
          'w-full flex items-center gap-3 p-3 text-left',
          hasDetails && 'hover:bg-surface-hover cursor-pointer'
        )}
        disabled={!hasDetails}
      >
        <span className="w-6 h-6 rounded bg-surface flex items-center justify-center text-xs font-mono text-muted-foreground">
          {index + 1}
        </span>
        <code className="flex-1 text-sm font-mono text-foreground break-all">
          {statement.statement}
        </code>
        {hasDetails && (
          <>
            <TruthValueInline
              frequency={statement.frequency!}
              confidence={statement.confidence!}
            />
            {isExpanded ? (
              <ChevronDown className="w-4 h-4 text-muted-foreground" />
            ) : (
              <ChevronRight className="w-4 h-4 text-muted-foreground" />
            )}
          </>
        )}
      </button>

      {isExpanded && hasDetails && (
        <div className="px-3 pb-3 pt-0 border-t border-border">
          <div className="flex items-start gap-2 mt-3">
            <Lightbulb className="w-4 h-4 text-warning mt-0.5" />
            <div>
              <p className="text-sm text-muted-foreground">Human-readable:</p>
              <p className="text-sm text-foreground mt-1">
                {humanizeNarsese(statement.statement)}
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export function NarsPanel({
  statements,
  title = 'NARS Reasoning',
  showExplanation = true,
  intent,
  className,
}: NarsPanelProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  // Parse raw statement strings if needed
  const parsedStatements = statements.map((s) =>
    typeof s === 'string' ? parseNarsese(s as unknown as string) : s
  );

  return (
    <div className={cn('bg-surface rounded-lg border border-border', className)}>
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between p-4 hover:bg-surface-hover transition-colors"
      >
        <div className="flex items-center gap-3">
          <div className="p-2 bg-accent-600/20 rounded-lg">
            <Brain className="w-5 h-5 text-accent-400" />
          </div>
          <div className="text-left">
            <h3 className="font-semibold text-foreground">{title}</h3>
            <p className="text-xs text-muted-foreground">
              {parsedStatements.length} statement{parsedStatements.length !== 1 ? 's' : ''}
              {intent && (
                <span className="ml-2">
                  <ArrowRight className="inline w-3 h-3" /> Intent: <span className="text-primary-400">{intent}</span>
                </span>
              )}
            </p>
          </div>
        </div>
        {isExpanded ? (
          <ChevronDown className="w-5 h-5 text-muted-foreground" />
        ) : (
          <ChevronRight className="w-5 h-5 text-muted-foreground" />
        )}
      </button>

      {isExpanded && (
        <div className="px-4 pb-4 space-y-4">
          {showExplanation && (
            <div className="p-3 bg-primary-600/10 border border-primary-600/20 rounded-lg">
              <p className="text-sm text-muted-foreground">
                <span className="font-medium text-foreground">Neural proposes, symbolic disposes:</span>{' '}
                The LLM translates your query to Narsese. NARS performs symbolic reasoning with
                truth-values (frequency × confidence) to filter results and mitigate hallucinations.
              </p>
            </div>
          )}

          <div className="space-y-2">
            {parsedStatements.map((statement, index) => (
              <StatementRow key={index} statement={statement} index={index} />
            ))}
          </div>

          {parsedStatements.length === 0 && (
            <div className="text-center py-4 text-muted-foreground text-sm">
              No Narsese statements available
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/**
 * Compact version for inline display in results.
 */
export function NarsStatementBadge({ statement }: { statement: string }) {
  const parsed = parseNarsese(statement);

  return (
    <div className="inline-flex items-center gap-2 px-2 py-1 bg-surface-elevated rounded border border-border">
      <Brain className="w-3 h-3 text-accent-400" />
      <code className="text-xs font-mono text-foreground">{parsed.statement}</code>
      {parsed.frequency !== undefined && parsed.confidence !== undefined && (
        <TruthValueInline
          frequency={parsed.frequency}
          confidence={parsed.confidence}
          className="ml-1"
        />
      )}
    </div>
  );
}
