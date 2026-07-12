# Pagination

A component for navigating between pages of content.

## When to Use

Use Pagination when:
- Displaying large datasets split across multiple pages
- Users need to jump to specific pages
- Building data tables, search results, or product listings
- You want to show users their position in a dataset

## Props

| Prop           | Type                                      | Default     | Description                                      |
|----------------|-------------------------------------------|-------------|--------------------------------------------------|
| `count`        | `number`                                  | -           | **Required.** Total number of pages              |
| `page`         | `number`                                  | -           | Current page number (1-indexed, controlled)      |
| `defaultPage`  | `number`                                  | `1`         | Default page number (uncontrolled)               |
| `onChange`     | `(event, page: number) => void`           | -           | Callback fired when page changes                 |
| `siblingCount` | `number`                                  | `1`         | Number of pages to show on each side of current  |
| `boundaryCount`| `number`                                  | `1`         | Number of pages to show at start and end         |
| `disabled`     | `boolean`                                 | `false`     | If true, pagination is disabled                  |
| `hideFirstLast`| `boolean`                                 | `false`     | If true, hide first/last page buttons            |
| `hidePrevNext` | `boolean`                                 | `false`     | If true, hide previous/next page buttons         |
| `size`         | `'small' \| 'medium' \| 'large'`          | `'medium'`  | The size of the pagination                       |
| `color`        | `'primary' \| 'secondary' \| 'accent' \| 'success' \| 'error'` | `'primary'` | The color of active page |
| `className`    | `string`                                  | -           | Additional CSS classes                           |

## Examples

### Basic Usage

```tsx
import { Pagination } from '@nest/components';
import { useState } from 'react';

const [page, setPage] = useState(1);

<Pagination count={10} page={page} onChange={(e, p) => setPage(p)} />
```

### Uncontrolled

```tsx
import { Pagination } from '@nest/components';

<Pagination count={10} defaultPage={1} />
```

### With Many Pages

```tsx
import { Pagination } from '@nest/components';

// Shows ellipsis for skipped page ranges
<Pagination count={100} siblingCount={2} boundaryCount={2} defaultPage={50} />
```

### Sizes

```tsx
import { Pagination } from '@nest/components';

<Pagination count={10} size="small" />
<Pagination count={10} size="medium" />
<Pagination count={10} size="large" />
```

### Colors

```tsx
import { Pagination } from '@nest/components';

<Pagination count={10} color="primary" />
<Pagination count={10} color="secondary" />
<Pagination count={10} color="success" />
<Pagination count={10} color="error" />
```

### Hidden Controls

```tsx
import { Pagination } from '@nest/components';

// Hide first/last buttons
<Pagination count={10} hideFirstLast />

// Hide prev/next buttons
<Pagination count={10} hidePrevNext />

// Hide all navigation buttons
<Pagination count={10} hideFirstLast hidePrevNext />
```

### Disabled State

```tsx
import { Pagination } from '@nest/components';

<Pagination count={10} disabled />
```

### With Data Table

```tsx
import { useState } from 'react';
import { Pagination } from '@nest/components';

function UserTable() {
  const [page, setPage] = useState(1);
  const itemsPerPage = 10;
  const totalItems = 100;
  const totalPages = Math.ceil(totalItems / itemsPerPage);

  return (
    <div className="border rounded-lg overflow-hidden">
      <table className="w-full">
        <thead className="bg-nest-muted/50">
          <tr>
            <th className="px-4 py-2">Name</th>
            <th className="px-4 py-2">Email</th>
            <th className="px-4 py-2">Role</th>
          </tr>
        </thead>
        <tbody className="divide-y">
          {/* Table rows */}
        </tbody>
      </table>
      <div className="flex items-center justify-between px-4 py-3 border-t">
        <p className="text-sm text-nest-muted">
          Showing {(page - 1) * itemsPerPage + 1}-{Math.min(page * itemsPerPage, totalItems)} of {totalItems}
        </p>
        <Pagination count={totalPages} page={page} onChange={(e, p) => setPage(p)} />
      </div>
    </div>
  );
}
```

### Custom Sibling and Boundary Counts

```tsx
import { Pagination } from '@nest/components';

// Show more pages around current page
<Pagination 
  count={50} 
  siblingCount={3} 
  boundaryCount={2} 
  defaultPage={25} 
/>
```

## Accessibility

- Pagination uses `role="navigation"` for screen readers
- Each page button has `aria-label` indicating the page number
- Current page has `aria-current="page"`
- Navigation buttons (first, prev, next, last) have descriptive `aria-label`
- Keyboard navigation: Tab between buttons, Enter/Space to activate

## Keyboard Navigation

| Key         | Action                    |
|-------------|---------------------------|
| Tab         | Move between buttons      |
| Enter/Space | Activate focused button   |

## Tips

- Use `siblingCount` and `boundaryCount` to control how many page numbers are shown
- For mobile, consider using `hideFirstLast` and `hidePrevNext` to save space
- Always pair pagination with text indicating current position (e.g., "Showing 1-10 of 100")
- For infinite scroll or "Load More" patterns, consider alternatives to pagination
