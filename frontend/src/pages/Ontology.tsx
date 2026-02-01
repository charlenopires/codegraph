import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  ChevronDown,
  ChevronRight,
  Search,
  Layout,
  Navigation,
  FormInput,
  MousePointer,
  LayoutGrid,
  Bell,
  Layers,
  Image,
  Type,
  MoreHorizontal,
  ExternalLink,
} from 'lucide-react';
import {
  ONTOLOGY_GROUPS,
  DESIGN_SYSTEMS,
  type OntologyGroup,
  type OntologyCategory,
  getTotalCategoryCount,
} from '@/data/ontology';
import { cn } from '@/lib/utils';

const GROUP_ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  layout: Layout,
  navigation: Navigation,
  forms: FormInput,
  actions: MousePointer,
  display: LayoutGrid,
  feedback: Bell,
  overlay: Layers,
  media: Image,
  typography: Type,
  other: MoreHorizontal,
};

function CategoryCard({
  category,
}: {
  category: OntologyCategory;
}) {
  const [isExpanded, setIsExpanded] = useState(false);
  const navigate = useNavigate();

  const handleQueryClick = () => {
    navigate(`/query?category=${category.id}`);
  };

  const handleGraphClick = () => {
    navigate(`/graph?category=${category.id}`);
  };

  return (
    <div className="bg-surface-elevated rounded-lg border border-border overflow-hidden">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between p-4 hover:bg-surface-hover transition-colors text-left"
      >
        <div>
          <h3 className="font-medium text-foreground">{category.name}</h3>
          <p className="text-sm text-muted-foreground mt-0.5">{category.description}</p>
        </div>
        {isExpanded ? (
          <ChevronDown className="w-5 h-5 text-muted-foreground flex-shrink-0" />
        ) : (
          <ChevronRight className="w-5 h-5 text-muted-foreground flex-shrink-0" />
        )}
      </button>

      {isExpanded && (
        <div className="px-4 pb-4 space-y-4 border-t border-border pt-4">
          {/* Examples */}
          <div>
            <h4 className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
              Examples
            </h4>
            <div className="flex flex-wrap gap-2">
              {category.examples.map((example) => (
                <span
                  key={example}
                  className="px-2 py-1 bg-surface rounded text-xs text-foreground"
                >
                  {example}
                </span>
              ))}
            </div>
          </div>

          {/* Design System Variants */}
          <div>
            <h4 className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
              Design System Variants
            </h4>
            <div className="space-y-2">
              {Object.entries(category.designSystemVariants).map(([system, component]) => {
                const ds = DESIGN_SYSTEMS.find((d) => d.id === system);
                return (
                  <div
                    key={system}
                    className="flex items-center gap-3 text-sm"
                  >
                    <span
                      className="w-3 h-3 rounded-full flex-shrink-0"
                      style={{ backgroundColor: ds?.color || '#6b7280' }}
                    />
                    <span className="w-24 text-muted-foreground">{ds?.name || system}</span>
                    <code className="text-xs bg-surface px-2 py-0.5 rounded text-foreground">
                      {component}
                    </code>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-2 pt-2">
            <button
              onClick={handleQueryClick}
              className="flex items-center gap-2 px-3 py-1.5 bg-primary-600/20 text-primary-400 rounded text-sm hover:bg-primary-600/30 transition-colors"
            >
              <Search className="w-4 h-4" />
              Search
            </button>
            <button
              onClick={handleGraphClick}
              className="flex items-center gap-2 px-3 py-1.5 bg-accent-600/20 text-accent-400 rounded text-sm hover:bg-accent-600/30 transition-colors"
            >
              <ExternalLink className="w-4 h-4" />
              View in Graph
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function GroupSection({ group }: { group: OntologyGroup }) {
  const [isExpanded, setIsExpanded] = useState(true);
  const Icon = GROUP_ICONS[group.id] || MoreHorizontal;

  return (
    <div className="space-y-3">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center gap-3 p-3 bg-surface rounded-lg border border-border hover:bg-surface-hover transition-colors"
      >
        <div className="p-2 bg-primary-600/20 rounded-lg">
          <Icon className="w-5 h-5 text-primary-400" />
        </div>
        <div className="flex-1 text-left">
          <h2 className="font-semibold text-foreground">{group.name}</h2>
          <p className="text-sm text-muted-foreground">
            {group.description} • {group.categories.length} categories
          </p>
        </div>
        {isExpanded ? (
          <ChevronDown className="w-5 h-5 text-muted-foreground" />
        ) : (
          <ChevronRight className="w-5 h-5 text-muted-foreground" />
        )}
      </button>

      {isExpanded && (
        <div className="pl-4 space-y-2">
          {group.categories.map((category) => (
            <CategoryCard key={category.id} category={category} />
          ))}
        </div>
      )}
    </div>
  );
}

export function OntologyPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedDesignSystem, setSelectedDesignSystem] = useState<string | null>(null);

  // Filter categories based on search
  const filteredGroups = ONTOLOGY_GROUPS.map((group) => ({
    ...group,
    categories: group.categories.filter((cat) => {
      const matchesSearch =
        !searchQuery ||
        cat.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        cat.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        cat.examples.some((ex) => ex.toLowerCase().includes(searchQuery.toLowerCase()));

      const matchesDesignSystem =
        !selectedDesignSystem ||
        Object.keys(cat.designSystemVariants).includes(selectedDesignSystem);

      return matchesSearch && matchesDesignSystem;
    }),
  })).filter((group) => group.categories.length > 0);

  const totalVisible = filteredGroups.reduce(
    (sum, group) => sum + group.categories.length,
    0
  );

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground">Ontology Explorer</h1>
        <p className="text-muted-foreground mt-1">
          Browse {getTotalCategoryCount()} UI component categories across {ONTOLOGY_GROUPS.length} groups
        </p>
      </div>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search categories..."
            className="w-full pl-10 pr-4 py-2 bg-surface border border-border rounded-lg text-foreground placeholder-muted-foreground focus:ring-2 focus:ring-primary-500 focus:border-transparent"
          />
        </div>

        <div className="flex gap-2 flex-wrap">
          <button
            onClick={() => setSelectedDesignSystem(null)}
            className={cn(
              'px-3 py-2 rounded-lg text-sm font-medium transition-colors',
              !selectedDesignSystem
                ? 'bg-primary-600 text-white'
                : 'bg-surface border border-border text-foreground hover:bg-surface-hover'
            )}
          >
            All
          </button>
          {DESIGN_SYSTEMS.map((ds) => (
            <button
              key={ds.id}
              onClick={() =>
                setSelectedDesignSystem(selectedDesignSystem === ds.id ? null : ds.id)
              }
              className={cn(
                'flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
                selectedDesignSystem === ds.id
                  ? 'bg-primary-600 text-white'
                  : 'bg-surface border border-border text-foreground hover:bg-surface-hover'
              )}
            >
              <span
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: ds.color }}
              />
              {ds.name}
            </button>
          ))}
        </div>
      </div>

      {/* Results count */}
      {(searchQuery || selectedDesignSystem) && (
        <p className="text-sm text-muted-foreground">
          Showing {totalVisible} of {getTotalCategoryCount()} categories
        </p>
      )}

      {/* Category Groups */}
      <div className="space-y-6">
        {filteredGroups.map((group) => (
          <GroupSection key={group.id} group={group} />
        ))}
      </div>

      {/* Empty state */}
      {filteredGroups.length === 0 && (
        <div className="text-center py-12 text-muted-foreground">
          <Search className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p className="text-lg font-medium">No categories found</p>
          <p className="text-sm mt-1">Try adjusting your search or filters</p>
        </div>
      )}
    </div>
  );
}
