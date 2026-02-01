import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  X,
  Search,
  ThumbsUp,
  ThumbsDown,
  Tag,
  Palette,
  Link2,
  Brain,
  Loader2,
} from 'lucide-react';
import type { GraphElement } from '@/api/types';
import { useGenerationStore } from '@/stores/generation';
import { toast } from '@/stores/toast';
import { TruthValue } from '@/components/reasoning/TruthValue';
import { cn } from '@/lib/utils';

interface ElementDetailProps {
  element: GraphElement;
  onClose: () => void;
}

const DESIGN_SYSTEM_COLORS: Record<string, string> = {
  'material-ui': '#2196f3',
  'tailwind': '#38bdf8',
  'chakra': '#319795',
  'bootstrap': '#7952b3',
  'ant-design': '#1890ff',
  'shadcn': '#ffffff',
  'custom': '#a855f7',
  'unknown': '#6b7280',
};

export function ElementDetail({ element, onClose }: ElementDetailProps) {
  const navigate = useNavigate();
  const { submitFeedback, submittingFeedback } = useGenerationStore();
  const [feedbackGiven, setFeedbackGiven] = useState<'thumbs_up' | 'thumbs_down' | null>(null);

  // Mock data for demonstration - in production this would come from the API
  const mockNarsese = [
    `<${element.name} --> ${element.category}>`,
    `<${element.name} --> [${element.design_system}]>`,
    `<${element.name} <-> UI_Component>`,
  ];

  const mockConfidence = 0.75;

  const handleFeedback = async (type: 'thumbs_up' | 'thumbs_down') => {
    try {
      await submitFeedback(element.id, type);
      setFeedbackGiven(type);
      toast.success(
        type === 'thumbs_up' ? 'Positive feedback recorded' : 'Negative feedback recorded',
        'Thank you for improving CodeGraph!'
      );
    } catch {
      toast.error('Failed to submit feedback', 'Please try again');
    }
  };

  const handleQuerySimilar = () => {
    onClose();
    navigate(`/query?q=${encodeURIComponent(`find ${element.category} similar to ${element.name}`)}`);
  };

  const designSystemColor = DESIGN_SYSTEM_COLORS[element.design_system] || DESIGN_SYSTEM_COLORS['unknown'];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative bg-surface rounded-xl border border-border shadow-2xl w-full max-w-lg max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-border">
          <div className="flex items-center gap-3">
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: designSystemColor }}
            />
            <h2 className="text-lg font-semibold text-foreground">{element.name}</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-surface-hover rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-muted-foreground" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {/* Basic info */}
          <div className="grid grid-cols-2 gap-4">
            <div className="flex items-center gap-3 p-3 bg-surface-elevated rounded-lg">
              <Tag className="w-5 h-5 text-primary-400" />
              <div>
                <p className="text-xs text-muted-foreground">Category</p>
                <p className="text-sm font-medium text-foreground">{element.category}</p>
              </div>
            </div>
            <div className="flex items-center gap-3 p-3 bg-surface-elevated rounded-lg">
              <Palette className="w-5 h-5 text-accent-400" />
              <div>
                <p className="text-xs text-muted-foreground">Design System</p>
                <p className="text-sm font-medium text-foreground">{element.design_system}</p>
              </div>
            </div>
          </div>

          {/* Connections */}
          <div className="p-3 bg-surface-elevated rounded-lg">
            <div className="flex items-center gap-2 mb-2">
              <Link2 className="w-4 h-4 text-warning" />
              <p className="text-sm font-medium text-foreground">Connections</p>
            </div>
            <p className="text-2xl font-bold text-foreground">{element.connections}</p>
            <p className="text-xs text-muted-foreground mt-1">
              Related elements in the knowledge graph
            </p>
          </div>

          {/* Confidence / Truth Value */}
          <div className="p-3 bg-surface-elevated rounded-lg">
            <div className="flex items-center gap-2 mb-3">
              <Brain className="w-4 h-4 text-success" />
              <p className="text-sm font-medium text-foreground">NARS Confidence</p>
            </div>
            <TruthValue
              frequency={mockConfidence}
              confidence={mockConfidence * 0.9}
              size="md"
              showLabels={true}
            />
          </div>

          {/* Narsese statements */}
          <div className="p-3 bg-surface-elevated rounded-lg">
            <div className="flex items-center gap-2 mb-3">
              <Brain className="w-4 h-4 text-accent-400" />
              <p className="text-sm font-medium text-foreground">Narsese Statements</p>
            </div>
            <div className="space-y-2">
              {mockNarsese.map((stmt, i) => (
                <code
                  key={i}
                  className="block text-xs font-mono text-muted-foreground bg-surface p-2 rounded"
                >
                  {stmt}
                </code>
              ))}
            </div>
          </div>

          {/* Feedback */}
          <div className="p-3 bg-surface-elevated rounded-lg">
            <p className="text-sm font-medium text-foreground mb-3">Rate this element</p>
            <div className="flex gap-2">
              <button
                onClick={() => handleFeedback('thumbs_up')}
                disabled={submittingFeedback.has(element.id) || feedbackGiven !== null}
                className={cn(
                  'flex-1 flex items-center justify-center gap-2 py-2 rounded-lg transition-colors',
                  feedbackGiven === 'thumbs_up'
                    ? 'bg-success/20 text-success border border-success/30'
                    : 'bg-surface hover:bg-surface-hover border border-border text-foreground',
                  'disabled:opacity-50 disabled:cursor-not-allowed'
                )}
              >
                {submittingFeedback.has(element.id) ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <ThumbsUp className="w-4 h-4" />
                )}
                Helpful
              </button>
              <button
                onClick={() => handleFeedback('thumbs_down')}
                disabled={submittingFeedback.has(element.id) || feedbackGiven !== null}
                className={cn(
                  'flex-1 flex items-center justify-center gap-2 py-2 rounded-lg transition-colors',
                  feedbackGiven === 'thumbs_down'
                    ? 'bg-error/20 text-error border border-error/30'
                    : 'bg-surface hover:bg-surface-hover border border-border text-foreground',
                  'disabled:opacity-50 disabled:cursor-not-allowed'
                )}
              >
                <ThumbsDown className="w-4 h-4" />
                Not helpful
              </button>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-border">
          <button
            onClick={handleQuerySimilar}
            className="w-full flex items-center justify-center gap-2 py-2.5 bg-primary-600 hover:bg-primary-700 text-white rounded-lg font-medium transition-colors"
          >
            <Search className="w-4 h-4" />
            Query Similar Elements
          </button>
        </div>
      </div>
    </div>
  );
}
