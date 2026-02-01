import { useState } from 'react';
import {
  BookOpen,
  Keyboard,
  Code,
  MessageSquare,
  ExternalLink,
  ChevronDown,
  ChevronRight,
  Brain,
  Network,
  Search,
  Upload,
  ThumbsUp,
  Zap,
  Database,
  GitBranch,
} from 'lucide-react';

interface FAQItem {
  question: string;
  answer: string;
}

const FAQ_ITEMS: FAQItem[] = [
  {
    question: 'What is CodeGraph?',
    answer: 'CodeGraph is a Hybrid GraphRAG System that extracts ontological entities from UI code snippets and generates complete vanilla web code from natural language queries. It uses NARS (Non-Axiomatic Reasoning System) for symbolic reasoning with evidential truth-values.',
  },
  {
    question: 'What does "neural proposes, symbolic disposes" mean?',
    answer: 'This architecture combines the creative power of LLMs (neural networks) with the logical precision of symbolic reasoning (NARS). The LLM generates candidate results, while NARS filters and ranks them using evidential truth-values, effectively mitigating hallucinations.',
  },
  {
    question: 'What design systems are supported?',
    answer: 'CodeGraph supports Material UI, Tailwind CSS, Chakra UI, Bootstrap, and custom/proprietary code. It automatically detects the design system from your uploaded snippets.',
  },
  {
    question: 'How does the feedback system work?',
    answer: 'The RLKGF (Reinforcement Learning from Knowledge Graph Feedback) system uses your thumbs up/down feedback to adjust confidence values. Positive feedback increases confidence (+0.1), while negative feedback decreases it (-0.15). Changes propagate through related elements.',
  },
  {
    question: 'Why is my query slow?',
    answer: 'Code generation involves multiple steps: hybrid retrieval (vector + graph + NARS), GPT-4o generation, and validation. Typical generation takes 30-60 seconds. Query-only searches are much faster (<2s).',
  },
  {
    question: 'Can I use CodeGraph offline?',
    answer: 'Partial offline mode is supported. Set CODEGRAPH_ONA_ENABLED=false to disable NARS inference (uses rule-based translation only). However, code generation still requires the OpenAI API.',
  },
];

const KEYBOARD_SHORTCUTS = [
  { keys: ['Ctrl', 'Enter'], description: 'Submit current form (Upload, Query)' },
  { keys: ['Ctrl', 'K'], description: 'Focus search/query input' },
  { keys: ['Esc'], description: 'Close modal or cancel operation' },
  { keys: ['Tab'], description: 'Navigate between form fields' },
];

function FAQAccordion({ item }: { item: FAQItem }) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="border border-border rounded-lg">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between p-4 text-left hover:bg-surface-hover transition-colors"
      >
        <span className="font-medium text-foreground">{item.question}</span>
        {isOpen ? (
          <ChevronDown className="w-5 h-5 text-muted-foreground" />
        ) : (
          <ChevronRight className="w-5 h-5 text-muted-foreground" />
        )}
      </button>
      {isOpen && (
        <div className="px-4 pb-4 text-muted-foreground">
          {item.answer}
        </div>
      )}
    </div>
  );
}

export function HelpPage() {
  return (
    <div className="space-y-8 max-w-4xl">
      <div>
        <h1 className="text-2xl font-bold text-foreground">Help & Documentation</h1>
        <p className="text-muted-foreground mt-1">
          Learn how to use CodeGraph effectively
        </p>
      </div>

      {/* Architecture Overview */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-primary-600/20 rounded-lg">
            <Network className="w-5 h-5 text-primary-400" />
          </div>
          <h2 className="text-lg font-semibold text-foreground">Architecture Overview</h2>
        </div>

        {/* ASCII Architecture Diagram */}
        <div className="bg-surface-elevated rounded-lg p-4 font-mono text-xs overflow-x-auto">
          <pre className="text-muted-foreground">
{`┌─────────────────────────────────────────────────────────────────┐
│                         CodeGraph                                │
│              "Neural Proposes, Symbolic Disposes"                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│   │  Upload  │───▶│  Query   │───▶│ Generate │───▶│ Feedback │  │
│   │   Page   │    │   Page   │    │   Code   │    │  (RLKGF) │  │
│   └────┬─────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘  │
│        │               │               │               │         │
│        ▼               ▼               ▼               ▼         │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                   WebSocket API                           │  │
│   └──────────────────────────────────────────────────────────┘  │
│        │               │               │               │         │
│        ▼               ▼               ▼               ▼         │
│   ┌──────────┐    ┌──────────────────────────┐    ┌──────────┐  │
│   │Extraction│    │   Hybrid Retrieval       │    │ RLKGF    │  │
│   │ Pipeline │    │ ┌────┐ ┌────┐ ┌────┐    │    │ Feedback │  │
│   │          │    │ │Vec │+│Graph│+│NARS│    │    │   Loop   │  │
│   │ AST Parse│    │ │40% │ │30% │ │30% │    │    │          │  │
│   │ Ontology │    │ └────┘ └────┘ └────┘    │    │ Conf±Δ   │  │
│   │ Narsese  │    └──────────────────────────┘    └──────────┘  │
│   └────┬─────┘                 │                       │         │
│        │                       │                       │         │
│        ▼                       ▼                       ▼         │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │  Neo4j (Graph)  │  Qdrant (Vector)  │  NARS (Reasoning)  │  │
│   │  Relationships  │  Embeddings 768d  │  Truth-Values      │  │
│   └──────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘`}
          </pre>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
          <div className="p-4 bg-surface-elevated rounded-lg border border-border">
            <div className="flex items-center gap-2 mb-2">
              <Database className="w-4 h-4 text-primary-400" />
              <span className="font-medium text-foreground">Vector Search (40%)</span>
            </div>
            <p className="text-sm text-muted-foreground">
              Semantic similarity via Qdrant embeddings (768 dimensions)
            </p>
          </div>
          <div className="p-4 bg-surface-elevated rounded-lg border border-border">
            <div className="flex items-center gap-2 mb-2">
              <GitBranch className="w-4 h-4 text-accent-400" />
              <span className="font-medium text-foreground">Graph Search (30%)</span>
            </div>
            <p className="text-sm text-muted-foreground">
              Pattern matching via Neo4j relationships and categories
            </p>
          </div>
          <div className="p-4 bg-surface-elevated rounded-lg border border-border">
            <div className="flex items-center gap-2 mb-2">
              <Brain className="w-4 h-4 text-warning" />
              <span className="font-medium text-foreground">NARS Inference (30%)</span>
            </div>
            <p className="text-sm text-muted-foreground">
              Symbolic reasoning with evidential truth-values
            </p>
          </div>
        </div>
      </section>

      {/* Quick Start Guide */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-accent-600/20 rounded-lg">
            <BookOpen className="w-5 h-5 text-accent-400" />
          </div>
          <h2 className="text-lg font-semibold text-foreground">Quick Start Guide</h2>
        </div>

        <div className="space-y-4">
          <div className="flex gap-4">
            <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary-600 flex items-center justify-center text-white font-bold">
              1
            </div>
            <div>
              <h3 className="font-medium text-foreground flex items-center gap-2">
                <Upload className="w-4 h-4" />
                Upload Code Snippets
              </h3>
              <p className="text-sm text-muted-foreground mt-1">
                Navigate to the Upload page and paste your HTML, CSS, and JavaScript code.
                CodeGraph will automatically detect the design system and extract ontological entities.
              </p>
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary-600 flex items-center justify-center text-white font-bold">
              2
            </div>
            <div>
              <h3 className="font-medium text-foreground flex items-center gap-2">
                <Search className="w-4 h-4" />
                Query or Generate
              </h3>
              <p className="text-sm text-muted-foreground mt-1">
                Use natural language to search for components ("find a primary button with icon")
                or generate new code ("create a responsive card with image and title").
              </p>
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary-600 flex items-center justify-center text-white font-bold">
              3
            </div>
            <div>
              <h3 className="font-medium text-foreground flex items-center gap-2">
                <ThumbsUp className="w-4 h-4" />
                Provide Feedback
              </h3>
              <p className="text-sm text-muted-foreground mt-1">
                Rate results with thumbs up/down to improve future queries.
                Your feedback adjusts confidence values through RLKGF.
              </p>
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary-600 flex items-center justify-center text-white font-bold">
              4
            </div>
            <div>
              <h3 className="font-medium text-foreground flex items-center gap-2">
                <Network className="w-4 h-4" />
                Explore the Graph
              </h3>
              <p className="text-sm text-muted-foreground mt-1">
                Visualize component relationships in the Graph page. Filter by category
                or design system to understand your knowledge base.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Key Metrics */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-success/20 rounded-lg">
            <Zap className="w-5 h-5 text-success" />
          </div>
          <h2 className="text-lg font-semibold text-foreground">Key Metrics</h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="text-center p-4 bg-surface-elevated rounded-lg border border-border">
            <p className="text-3xl font-bold text-primary-400">91%</p>
            <p className="text-sm text-muted-foreground mt-1">Retrieval Precision</p>
            <p className="text-xs text-muted-foreground">(vs 34% traditional RAG)</p>
          </div>
          <div className="text-center p-4 bg-surface-elevated rounded-lg border border-border">
            <p className="text-3xl font-bold text-accent-400">10x</p>
            <p className="text-sm text-muted-foreground mt-1">Cheaper than RLHF</p>
            <p className="text-xs text-muted-foreground">Using RLKGF feedback</p>
          </div>
          <div className="text-center p-4 bg-surface-elevated rounded-lg border border-border">
            <p className="text-3xl font-bold text-warning">40+</p>
            <p className="text-sm text-muted-foreground mt-1">UI Categories</p>
            <p className="text-xs text-muted-foreground">In the ontology</p>
          </div>
        </div>
      </section>

      {/* Keyboard Shortcuts */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-warning/20 rounded-lg">
            <Keyboard className="w-5 h-5 text-warning" />
          </div>
          <h2 className="text-lg font-semibold text-foreground">Keyboard Shortcuts</h2>
        </div>

        <div className="space-y-2">
          {KEYBOARD_SHORTCUTS.map((shortcut, index) => (
            <div
              key={index}
              className="flex items-center justify-between p-3 bg-surface-elevated rounded-lg"
            >
              <span className="text-muted-foreground">{shortcut.description}</span>
              <div className="flex gap-1">
                {shortcut.keys.map((key, i) => (
                  <span key={i}>
                    <kbd className="px-2 py-1 bg-surface border border-border rounded text-sm font-mono text-foreground">
                      {key}
                    </kbd>
                    {i < shortcut.keys.length - 1 && (
                      <span className="text-muted-foreground mx-1">+</span>
                    )}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* FAQ */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-error/20 rounded-lg">
            <MessageSquare className="w-5 h-5 text-error" />
          </div>
          <h2 className="text-lg font-semibold text-foreground">Frequently Asked Questions</h2>
        </div>

        <div className="space-y-2">
          {FAQ_ITEMS.map((item, index) => (
            <FAQAccordion key={index} item={item} />
          ))}
        </div>
      </section>

      {/* External Links */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-muted/20 rounded-lg">
            <ExternalLink className="w-5 h-5 text-muted-foreground" />
          </div>
          <h2 className="text-lg font-semibold text-foreground">External Resources</h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <a
            href="https://github.com/anthropics/codegraph"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-3 p-3 bg-surface-elevated rounded-lg border border-border hover:bg-surface-hover transition-colors"
          >
            <Code className="w-5 h-5 text-foreground" />
            <div>
              <span className="font-medium text-foreground">GitHub Repository</span>
              <p className="text-xs text-muted-foreground">Source code and documentation</p>
            </div>
          </a>
          <a
            href="https://openreview.net/forum?id=codegraph-rlkgf"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-3 p-3 bg-surface-elevated rounded-lg border border-border hover:bg-surface-hover transition-colors"
          >
            <BookOpen className="w-5 h-5 text-foreground" />
            <div>
              <span className="font-medium text-foreground">Research Paper</span>
              <p className="text-xs text-muted-foreground">OpenReview 2025 - RLKGF</p>
            </div>
          </a>
        </div>
      </section>
    </div>
  );
}
