# Design System

> Extracted from: https://i.pinimg.com/1200x/d6/d3/47/d6d347e4706a94e19ac1f751c7846ee9.jpg
> Generated: 2026-02-01 03:09:48

All UI implementation MUST follow the design tokens defined below.

## CSS Custom Properties

```css
:root {

  /* Primary Colors */
  --color-primary-400: #FDB022;
  --color-primary-500: #F59E0B;

  /* Secondary Colors */
  --color-secondary-400: #FB7185;
  --color-secondary-500: #EF4444;

  /* Neutral Colors */
  --color-neutral-50: #F9FAFB;
  --color-neutral-100: #F3F4F6;
  --color-neutral-300: #D1D5DB;
  --color-neutral-500: #6B7280;
  --color-neutral-700: #374151;
  --color-neutral-800: #1F2937;
  --color-neutral-900: #111827;

  /* Semantic Colors */
  --color-success: #10B981;
  --color-warning: #F59E0B;
  --color-error: #EF4444;
  --color-info: #3B82F6;

  /* Spacing */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 12px;
  --spacing-lg: 16px;
  --spacing-xl: 20px;
  --spacing-2xl: 24px;
  --spacing-3xl: 32px;

  /* Border Radius */
  --radius-sm: 6px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --radius-xl: 20px;
  --radius-full: 9999px;

  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
  --shadow-md: 0 4px 6px rgba(0,0,0,0.07);
  --shadow-lg: 0 10px 15px rgba(0,0,0,0.1);

  /* Breakpoints */
  --breakpoint-mobile: 320px;
  --breakpoint-tablet: 768px;
  --breakpoint-desktop: 1024px;
  --breakpoint-large: 1280px;
}
```

## Color Palette

### Primary Colors

| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `--color-primary-400` | `#FDB022` | 253, 176, 34 | Golden accent, workout highlights |
| `--color-primary-500` | `#F59E0B` | 245, 158, 11 | Primary brand color, calories intake visualization |

### Secondary Colors

| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `--color-secondary-400` | `#FB7185` | 251, 113, 133 | Calories burned indicator |
| `--color-secondary-500` | `#EF4444` | 239, 68, 68 | Error states, deletion actions |

### Neutral Colors

| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `--color-neutral-50` | `#F9FAFB` | 249, 250, 251 | Background, card surfaces |
| `--color-neutral-100` | `#F3F4F6` | 243, 244, 246 | Secondary backgrounds |
| `--color-neutral-300` | `#D1D5DB` | 209, 213, 219 | Borders, dividers |
| `--color-neutral-500` | `#6B7280` | 107, 114, 128 | Secondary text, placeholders |
| `--color-neutral-700` | `#374151` | 55, 65, 81 | Primary text |
| `--color-neutral-800` | `#1F2937` | 31, 41, 55 | Dark surfaces, calendar |
| `--color-neutral-900` | `#111827` | 17, 24, 39 | Darkest elements, primary buttons |

### Semantic Colors

| Role | Hex |
|------|-----|
| success | `#10B981` |
| warning | `#F59E0B` |
| error | `#EF4444` |
| info | `#3B82F6` |

## Typography

### Font Families

| Family | Category | Weights | Usage |
|--------|----------|---------|-------|
| Inter | sans-serif | 400, 500, 600, 700 | Primary interface font |

### Type Scale

| Token | Size | Weight | Line Height |
|-------|------|--------|-------------|
| `heading-xl` | 24px | 600 | 1.3 |
| `heading-lg` | 20px | 600 | 1.3 |
| `heading-md` | 18px | 600 | 1.4 |
| `body-lg` | 16px | 400 | 1.5 |
| `body-md` | 14px | 400 | 1.5 |
| `body-sm` | 12px | 400 | 1.4 |
| `caption` | 11px | 400 | 1.3 |

## Spacing Scale

| Token | Value |
|-------|-------|
| `--spacing-xs` | 4px |
| `--spacing-sm` | 8px |
| `--spacing-md` | 12px |
| `--spacing-lg` | 16px |
| `--spacing-xl` | 20px |
| `--spacing-2xl` | 24px |
| `--spacing-3xl` | 32px |

## Border Radius

| Token | Value |
|-------|-------|
| `--radius-sm` | 6px |
| `--radius-md` | 12px |
| `--radius-lg` | 16px |
| `--radius-xl` | 20px |
| `--radius-full` | 9999px |

## Shadows

| Token | Value |
|-------|-------|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.05)` |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.07)` |
| `--shadow-lg` | `0 10px 15px rgba(0,0,0,0.1)` |

## Breakpoints

| Name | Min Width |
|------|-----------|
| mobile | 320px |
| tablet | 768px |
| desktop | 1024px |
| large | 1280px |

## Components

### Button

Action buttons with various states

**Variants:** primary, secondary, ghost

**States:** default, hover, disabled

### Card

Content containers with rounded corners

**Variants:** elevated, flat, dark

**States:** default, hover

### Calendar

Training days calendar component

**Variants:** compact

**States:** current, done, scheduled

### ProgressBar

Linear progress indicators

**Variants:** default, multi-segment

**States:** in-progress, complete

### CircularProgress

Circular progress indicators

**Variants:** steps, goals

**States:** in-progress, complete

### Avatar

User profile images

**Variants:** small, medium, large

**States:** default, online

### SearchInput

Search field with icon

**Variants:** default

**States:** default, focused, disabled

### Sidebar

Navigation sidebar

**Variants:** collapsed, expanded

**States:** default

### HabitCard

Individual habit tracking cards

**Variants:** default

**States:** active, completed

### DataVisualization

Bubble chart for workout metrics

**Variants:** calories

**States:** default

