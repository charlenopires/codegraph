export interface OntologyCategory {
  id: string;
  name: string;
  description: string;
  examples: string[];
  designSystemVariants: {
    [key: string]: string;
  };
}

export interface OntologyGroup {
  id: string;
  name: string;
  icon: string;
  description: string;
  categories: OntologyCategory[];
}

export const DESIGN_SYSTEMS = [
  { id: 'material-ui', name: 'Material UI', color: '#2196f3' },
  { id: 'tailwind', name: 'Tailwind CSS', color: '#38bdf8' },
  { id: 'chakra', name: 'Chakra UI', color: '#319795' },
  { id: 'bootstrap', name: 'Bootstrap', color: '#7952b3' },
  { id: 'custom', name: 'Custom', color: '#a855f7' },
];

export const ONTOLOGY_GROUPS: OntologyGroup[] = [
  {
    id: 'layout',
    name: 'Layout',
    icon: 'Layout',
    description: 'Container and structural components',
    categories: [
      {
        id: 'container',
        name: 'Container',
        description: 'Wrapper component that centers content with max-width',
        examples: ['max-width container', 'fluid container', 'responsive wrapper'],
        designSystemVariants: {
          'material-ui': 'Container',
          tailwind: 'container mx-auto',
          chakra: 'Container',
          bootstrap: 'container',
        },
      },
      {
        id: 'grid',
        name: 'Grid',
        description: 'Two-dimensional layout system with rows and columns',
        examples: ['12-column grid', 'auto-fit grid', 'responsive grid'],
        designSystemVariants: {
          'material-ui': 'Grid container',
          tailwind: 'grid grid-cols-*',
          chakra: 'SimpleGrid / Grid',
          bootstrap: 'row/col',
        },
      },
      {
        id: 'flex',
        name: 'Flex',
        description: 'One-dimensional flexible layout',
        examples: ['flex row', 'flex column', 'justify-between'],
        designSystemVariants: {
          'material-ui': 'Box display="flex"',
          tailwind: 'flex flex-row/col',
          chakra: 'Flex / HStack / VStack',
          bootstrap: 'd-flex',
        },
      },
      {
        id: 'stack',
        name: 'Stack',
        description: 'Vertical or horizontal stack with consistent spacing',
        examples: ['vertical stack', 'horizontal stack', 'responsive stack'],
        designSystemVariants: {
          'material-ui': 'Stack',
          tailwind: 'space-y-* / space-x-*',
          chakra: 'Stack / VStack / HStack',
          bootstrap: 'vstack / hstack',
        },
      },
    ],
  },
  {
    id: 'navigation',
    name: 'Navigation',
    icon: 'Navigation',
    description: 'Navigation and wayfinding components',
    categories: [
      {
        id: 'navigation',
        name: 'Navigation',
        description: 'Primary navigation container',
        examples: ['navbar', 'top nav', 'responsive nav'],
        designSystemVariants: {
          'material-ui': 'AppBar',
          tailwind: 'nav',
          chakra: 'Flex as nav',
          bootstrap: 'navbar',
        },
      },
      {
        id: 'menu',
        name: 'Menu',
        description: 'Dropdown or popup menu with items',
        examples: ['dropdown menu', 'context menu', 'action menu'],
        designSystemVariants: {
          'material-ui': 'Menu / MenuList',
          tailwind: 'dropdown (custom)',
          chakra: 'Menu',
          bootstrap: 'dropdown-menu',
        },
      },
      {
        id: 'breadcrumb',
        name: 'Breadcrumb',
        description: 'Navigation path showing current location',
        examples: ['breadcrumb trail', 'path indicator'],
        designSystemVariants: {
          'material-ui': 'Breadcrumbs',
          tailwind: 'breadcrumb (custom)',
          chakra: 'Breadcrumb',
          bootstrap: 'breadcrumb',
        },
      },
      {
        id: 'tabs',
        name: 'Tabs',
        description: 'Tabbed interface for switching views',
        examples: ['horizontal tabs', 'vertical tabs', 'scrollable tabs'],
        designSystemVariants: {
          'material-ui': 'Tabs / Tab',
          tailwind: 'tabs (custom)',
          chakra: 'Tabs',
          bootstrap: 'nav-tabs',
        },
      },
      {
        id: 'pagination',
        name: 'Pagination',
        description: 'Page navigation for multi-page content',
        examples: ['numbered pagination', 'prev/next', 'infinite scroll'],
        designSystemVariants: {
          'material-ui': 'Pagination',
          tailwind: 'pagination (custom)',
          chakra: 'Pagination (custom)',
          bootstrap: 'pagination',
        },
      },
      {
        id: 'sidebar',
        name: 'Sidebar',
        description: 'Side navigation or content panel',
        examples: ['left sidebar', 'collapsible sidebar', 'navigation drawer'],
        designSystemVariants: {
          'material-ui': 'Drawer',
          tailwind: 'sidebar (custom)',
          chakra: 'Drawer',
          bootstrap: 'offcanvas',
        },
      },
    ],
  },
  {
    id: 'forms',
    name: 'Forms',
    icon: 'FormInput',
    description: 'Form controls and input components',
    categories: [
      {
        id: 'form',
        name: 'Form',
        description: 'Form container with validation',
        examples: ['login form', 'signup form', 'contact form'],
        designSystemVariants: {
          'material-ui': 'form element',
          tailwind: 'form',
          chakra: 'form',
          bootstrap: 'form',
        },
      },
      {
        id: 'input',
        name: 'Input',
        description: 'Text input field',
        examples: ['text input', 'password input', 'search input'],
        designSystemVariants: {
          'material-ui': 'TextField',
          tailwind: 'input',
          chakra: 'Input',
          bootstrap: 'form-control',
        },
      },
      {
        id: 'textarea',
        name: 'Textarea',
        description: 'Multi-line text input',
        examples: ['message textarea', 'comment box', 'description field'],
        designSystemVariants: {
          'material-ui': 'TextField multiline',
          tailwind: 'textarea',
          chakra: 'Textarea',
          bootstrap: 'form-control textarea',
        },
      },
      {
        id: 'select',
        name: 'Select',
        description: 'Dropdown selection control',
        examples: ['country select', 'options dropdown', 'multi-select'],
        designSystemVariants: {
          'material-ui': 'Select',
          tailwind: 'select',
          chakra: 'Select',
          bootstrap: 'form-select',
        },
      },
      {
        id: 'checkbox',
        name: 'Checkbox',
        description: 'Boolean checkbox input',
        examples: ['terms checkbox', 'feature toggle', 'multi-select list'],
        designSystemVariants: {
          'material-ui': 'Checkbox',
          tailwind: 'checkbox',
          chakra: 'Checkbox',
          bootstrap: 'form-check-input',
        },
      },
      {
        id: 'radio',
        name: 'Radio',
        description: 'Single selection from options',
        examples: ['radio group', 'option selector', 'plan picker'],
        designSystemVariants: {
          'material-ui': 'Radio',
          tailwind: 'radio',
          chakra: 'Radio',
          bootstrap: 'form-check-input radio',
        },
      },
      {
        id: 'switch',
        name: 'Switch',
        description: 'Toggle switch control',
        examples: ['on/off switch', 'setting toggle', 'theme switch'],
        designSystemVariants: {
          'material-ui': 'Switch',
          tailwind: 'toggle (custom)',
          chakra: 'Switch',
          bootstrap: 'form-check-input switch',
        },
      },
      {
        id: 'slider',
        name: 'Slider',
        description: 'Range slider input',
        examples: ['volume slider', 'price range', 'progress control'],
        designSystemVariants: {
          'material-ui': 'Slider',
          tailwind: 'range',
          chakra: 'Slider',
          bootstrap: 'form-range',
        },
      },
      {
        id: 'datepicker',
        name: 'DatePicker',
        description: 'Date selection control',
        examples: ['date picker', 'date range', 'calendar picker'],
        designSystemVariants: {
          'material-ui': 'DatePicker',
          tailwind: 'datepicker (plugin)',
          chakra: 'DatePicker (plugin)',
          bootstrap: 'datepicker (plugin)',
        },
      },
      {
        id: 'autocomplete',
        name: 'Autocomplete',
        description: 'Input with suggestions',
        examples: ['search autocomplete', 'tag input', 'combobox'],
        designSystemVariants: {
          'material-ui': 'Autocomplete',
          tailwind: 'combobox (custom)',
          chakra: 'AutoComplete (plugin)',
          bootstrap: 'typeahead (plugin)',
        },
      },
    ],
  },
  {
    id: 'actions',
    name: 'Actions',
    icon: 'MousePointer',
    description: 'Interactive action components',
    categories: [
      {
        id: 'button',
        name: 'Button',
        description: 'Clickable action button',
        examples: ['primary button', 'outlined button', 'icon button'],
        designSystemVariants: {
          'material-ui': 'Button',
          tailwind: 'btn (custom)',
          chakra: 'Button',
          bootstrap: 'btn',
        },
      },
      {
        id: 'link',
        name: 'Link',
        description: 'Clickable text link',
        examples: ['text link', 'nav link', 'external link'],
        designSystemVariants: {
          'material-ui': 'Link',
          tailwind: 'a',
          chakra: 'Link',
          bootstrap: 'nav-link / a',
        },
      },
      {
        id: 'iconbutton',
        name: 'IconButton',
        description: 'Icon-only action button',
        examples: ['close button', 'menu trigger', 'action icon'],
        designSystemVariants: {
          'material-ui': 'IconButton',
          tailwind: 'icon button (custom)',
          chakra: 'IconButton',
          bootstrap: 'btn with icon',
        },
      },
      {
        id: 'fab',
        name: 'FAB',
        description: 'Floating action button',
        examples: ['add FAB', 'action FAB', 'extended FAB'],
        designSystemVariants: {
          'material-ui': 'Fab',
          tailwind: 'fab (custom)',
          chakra: 'IconButton + position',
          bootstrap: 'btn fixed position',
        },
      },
    ],
  },
  {
    id: 'display',
    name: 'Display',
    icon: 'LayoutGrid',
    description: 'Content display components',
    categories: [
      {
        id: 'card',
        name: 'Card',
        description: 'Content card with surface elevation',
        examples: ['profile card', 'product card', 'info card'],
        designSystemVariants: {
          'material-ui': 'Card',
          tailwind: 'card (custom)',
          chakra: 'Card',
          bootstrap: 'card',
        },
      },
      {
        id: 'list',
        name: 'List',
        description: 'Vertical list of items',
        examples: ['item list', 'menu list', 'navigation list'],
        designSystemVariants: {
          'material-ui': 'List / ListItem',
          tailwind: 'ul/li',
          chakra: 'List',
          bootstrap: 'list-group',
        },
      },
      {
        id: 'table',
        name: 'Table',
        description: 'Tabular data display',
        examples: ['data table', 'sortable table', 'responsive table'],
        designSystemVariants: {
          'material-ui': 'Table',
          tailwind: 'table',
          chakra: 'Table',
          bootstrap: 'table',
        },
      },
      {
        id: 'avatar',
        name: 'Avatar',
        description: 'User profile image or initials',
        examples: ['user avatar', 'avatar group', 'avatar with status'],
        designSystemVariants: {
          'material-ui': 'Avatar',
          tailwind: 'avatar (custom)',
          chakra: 'Avatar',
          bootstrap: 'avatar (custom)',
        },
      },
      {
        id: 'badge',
        name: 'Badge',
        description: 'Small count or status indicator',
        examples: ['notification badge', 'status badge', 'count badge'],
        designSystemVariants: {
          'material-ui': 'Badge',
          tailwind: 'badge (custom)',
          chakra: 'Badge',
          bootstrap: 'badge',
        },
      },
      {
        id: 'chip',
        name: 'Chip',
        description: 'Compact element for input or display',
        examples: ['filter chip', 'tag chip', 'action chip'],
        designSystemVariants: {
          'material-ui': 'Chip',
          tailwind: 'chip (custom)',
          chakra: 'Tag',
          bootstrap: 'badge pill',
        },
      },
      {
        id: 'tag',
        name: 'Tag',
        description: 'Label or category indicator',
        examples: ['category tag', 'status tag', 'label tag'],
        designSystemVariants: {
          'material-ui': 'Chip variant',
          tailwind: 'tag (custom)',
          chakra: 'Tag',
          bootstrap: 'badge',
        },
      },
      {
        id: 'accordion',
        name: 'Accordion',
        description: 'Expandable content sections',
        examples: ['FAQ accordion', 'settings accordion', 'content sections'],
        designSystemVariants: {
          'material-ui': 'Accordion',
          tailwind: 'disclosure (custom)',
          chakra: 'Accordion',
          bootstrap: 'accordion',
        },
      },
    ],
  },
  {
    id: 'feedback',
    name: 'Feedback',
    icon: 'Bell',
    description: 'User feedback and status components',
    categories: [
      {
        id: 'alert',
        name: 'Alert',
        description: 'Status message banner',
        examples: ['success alert', 'error alert', 'warning alert'],
        designSystemVariants: {
          'material-ui': 'Alert',
          tailwind: 'alert (custom)',
          chakra: 'Alert',
          bootstrap: 'alert',
        },
      },
      {
        id: 'toast',
        name: 'Toast',
        description: 'Brief notification message',
        examples: ['success toast', 'error toast', 'action toast'],
        designSystemVariants: {
          'material-ui': 'Snackbar',
          tailwind: 'toast (custom)',
          chakra: 'Toast',
          bootstrap: 'toast',
        },
      },
      {
        id: 'snackbar',
        name: 'Snackbar',
        description: 'Brief message with optional action',
        examples: ['undo snackbar', 'confirm snackbar', 'info snackbar'],
        designSystemVariants: {
          'material-ui': 'Snackbar',
          tailwind: 'snackbar (custom)',
          chakra: 'Toast',
          bootstrap: 'toast',
        },
      },
      {
        id: 'progress',
        name: 'Progress',
        description: 'Progress indicator bar',
        examples: ['linear progress', 'determinate progress', 'buffer progress'],
        designSystemVariants: {
          'material-ui': 'LinearProgress',
          tailwind: 'progress (custom)',
          chakra: 'Progress',
          bootstrap: 'progress',
        },
      },
      {
        id: 'spinner',
        name: 'Spinner',
        description: 'Loading spinner animation',
        examples: ['loading spinner', 'circular progress', 'indeterminate loader'],
        designSystemVariants: {
          'material-ui': 'CircularProgress',
          tailwind: 'animate-spin',
          chakra: 'Spinner',
          bootstrap: 'spinner-border',
        },
      },
      {
        id: 'skeleton',
        name: 'Skeleton',
        description: 'Content placeholder during loading',
        examples: ['text skeleton', 'card skeleton', 'list skeleton'],
        designSystemVariants: {
          'material-ui': 'Skeleton',
          tailwind: 'skeleton (custom)',
          chakra: 'Skeleton',
          bootstrap: 'placeholder',
        },
      },
    ],
  },
  {
    id: 'overlay',
    name: 'Overlay',
    icon: 'Layers',
    description: 'Modal and overlay components',
    categories: [
      {
        id: 'modal',
        name: 'Modal',
        description: 'Dialog window overlay',
        examples: ['confirm modal', 'form modal', 'fullscreen modal'],
        designSystemVariants: {
          'material-ui': 'Modal / Dialog',
          tailwind: 'modal (custom)',
          chakra: 'Modal',
          bootstrap: 'modal',
        },
      },
      {
        id: 'dialog',
        name: 'Dialog',
        description: 'Structured dialog with actions',
        examples: ['alert dialog', 'confirm dialog', 'form dialog'],
        designSystemVariants: {
          'material-ui': 'Dialog',
          tailwind: 'dialog (custom)',
          chakra: 'AlertDialog',
          bootstrap: 'modal',
        },
      },
      {
        id: 'drawer',
        name: 'Drawer',
        description: 'Sliding panel from edge',
        examples: ['navigation drawer', 'filter drawer', 'detail drawer'],
        designSystemVariants: {
          'material-ui': 'Drawer',
          tailwind: 'drawer (custom)',
          chakra: 'Drawer',
          bootstrap: 'offcanvas',
        },
      },
      {
        id: 'popover',
        name: 'Popover',
        description: 'Popup content container',
        examples: ['info popover', 'menu popover', 'form popover'],
        designSystemVariants: {
          'material-ui': 'Popover',
          tailwind: 'popover (custom)',
          chakra: 'Popover',
          bootstrap: 'popover',
        },
      },
      {
        id: 'tooltip',
        name: 'Tooltip',
        description: 'Hover information tooltip',
        examples: ['help tooltip', 'info tooltip', 'action tooltip'],
        designSystemVariants: {
          'material-ui': 'Tooltip',
          tailwind: 'tooltip (custom)',
          chakra: 'Tooltip',
          bootstrap: 'tooltip',
        },
      },
      {
        id: 'contextmenu',
        name: 'ContextMenu',
        description: 'Right-click context menu',
        examples: ['file context menu', 'editor context menu', 'action context menu'],
        designSystemVariants: {
          'material-ui': 'Menu (custom trigger)',
          tailwind: 'context menu (custom)',
          chakra: 'Menu (custom trigger)',
          bootstrap: 'dropdown (custom trigger)',
        },
      },
    ],
  },
  {
    id: 'media',
    name: 'Media',
    icon: 'Image',
    description: 'Media and visual components',
    categories: [
      {
        id: 'image',
        name: 'Image',
        description: 'Image display component',
        examples: ['responsive image', 'lazy image', 'image gallery'],
        designSystemVariants: {
          'material-ui': 'img / CardMedia',
          tailwind: 'img',
          chakra: 'Image',
          bootstrap: 'img-fluid',
        },
      },
      {
        id: 'video',
        name: 'Video',
        description: 'Video player component',
        examples: ['video player', 'embedded video', 'video thumbnail'],
        designSystemVariants: {
          'material-ui': 'video (custom)',
          tailwind: 'video',
          chakra: 'video (custom)',
          bootstrap: 'embed-responsive',
        },
      },
      {
        id: 'icon',
        name: 'Icon',
        description: 'SVG icon component',
        examples: ['system icon', 'action icon', 'decorative icon'],
        designSystemVariants: {
          'material-ui': 'SvgIcon / Icon',
          tailwind: 'heroicons/lucide',
          chakra: 'Icon',
          bootstrap: 'bi-* icons',
        },
      },
    ],
  },
  {
    id: 'typography',
    name: 'Typography',
    icon: 'Type',
    description: 'Text and typography components',
    categories: [
      {
        id: 'heading',
        name: 'Heading',
        description: 'Section heading text',
        examples: ['page title', 'section heading', 'card title'],
        designSystemVariants: {
          'material-ui': 'Typography variant="h*"',
          tailwind: 'text-* font-bold',
          chakra: 'Heading',
          bootstrap: 'h1-h6 / display-*',
        },
      },
      {
        id: 'text',
        name: 'Text',
        description: 'Body text content',
        examples: ['paragraph', 'body text', 'description'],
        designSystemVariants: {
          'material-ui': 'Typography',
          tailwind: 'text-* prose',
          chakra: 'Text',
          bootstrap: 'p / text-*',
        },
      },
      {
        id: 'label',
        name: 'Label',
        description: 'Form field label',
        examples: ['input label', 'form label', 'field label'],
        designSystemVariants: {
          'material-ui': 'InputLabel',
          tailwind: 'label',
          chakra: 'FormLabel',
          bootstrap: 'form-label',
        },
      },
    ],
  },
  {
    id: 'other',
    name: 'Other',
    icon: 'MoreHorizontal',
    description: 'Utility and miscellaneous components',
    categories: [
      {
        id: 'divider',
        name: 'Divider',
        description: 'Content separator line',
        examples: ['horizontal divider', 'vertical divider', 'section divider'],
        designSystemVariants: {
          'material-ui': 'Divider',
          tailwind: 'border-t / divide-y',
          chakra: 'Divider',
          bootstrap: 'hr',
        },
      },
      {
        id: 'spacer',
        name: 'Spacer',
        description: 'Flexible space filler',
        examples: ['flex spacer', 'margin spacer', 'layout spacer'],
        designSystemVariants: {
          'material-ui': 'Box flexGrow={1}',
          tailwind: 'flex-1 / grow',
          chakra: 'Spacer',
          bootstrap: 'flex-grow-1',
        },
      },
    ],
  },
];

// Helper to get all categories as a flat list
export function getAllCategories(): OntologyCategory[] {
  return ONTOLOGY_GROUPS.flatMap((group) => group.categories);
}

// Helper to get category by ID
export function getCategoryById(id: string): OntologyCategory | undefined {
  return getAllCategories().find((cat) => cat.id === id);
}

// Helper to get total category count
export function getTotalCategoryCount(): number {
  return getAllCategories().length;
}
