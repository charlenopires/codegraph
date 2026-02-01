import { NavLink } from 'react-router-dom';
import {
  Upload,
  Search,
  Network,
  BarChart3,
  Settings,
  HelpCircle,
  BookOpen,
} from 'lucide-react';
import { cn } from '@/lib/utils';

interface NavItem {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
}

const mainNavItems: NavItem[] = [
  { to: '/upload', icon: Upload, label: 'Upload' },
  { to: '/query', icon: Search, label: 'Query' },
  { to: '/graph', icon: Network, label: 'Graph' },
  { to: '/metrics', icon: BarChart3, label: 'Metrics' },
  { to: '/ontology', icon: BookOpen, label: 'Ontology' },
];

const bottomNavItems: NavItem[] = [
  { to: '/settings', icon: Settings, label: 'Settings' },
  { to: '/help', icon: HelpCircle, label: 'Help' },
];

function NavItemLink({ item }: { item: NavItem }) {
  const Icon = item.icon;

  return (
    <NavLink
      to={item.to}
      className={({ isActive }) =>
        cn(
          'flex items-center gap-3 px-4 py-3 rounded-lg transition-all duration-200',
          'text-muted hover:text-foreground hover:bg-surface-hover',
          'focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus-visible:ring-offset-background',
          isActive && 'bg-primary-600/20 text-foreground border-l-2 border-primary-500'
        )
      }
    >
      <Icon className="w-5 h-5 flex-shrink-0" />
      <span className="font-medium">{item.label}</span>
    </NavLink>
  );
}

export function Sidebar() {
  return (
    <aside className="fixed left-0 top-0 h-screen w-64 bg-surface border-r border-border flex flex-col">
      {/* Logo */}
      <div className="px-6 py-5 border-b border-border">
        <h1 className="text-xl font-bold bg-gradient-to-r from-primary-400 to-accent-400 bg-clip-text text-transparent">
          CodeGraph
        </h1>
        <p className="text-xs text-muted-foreground mt-1">
          Knowledge-driven UI
        </p>
      </div>

      {/* Main navigation */}
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
        {mainNavItems.map((item) => (
          <NavItemLink key={item.to} item={item} />
        ))}
      </nav>

      {/* Bottom navigation */}
      <nav className="px-3 py-4 space-y-1 border-t border-border">
        {bottomNavItems.map((item) => (
          <NavItemLink key={item.to} item={item} />
        ))}
      </nav>
    </aside>
  );
}
