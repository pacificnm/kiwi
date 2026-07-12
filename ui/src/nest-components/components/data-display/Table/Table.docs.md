# Table

Components for displaying tabular data with support for headers, body rows, footers, and various cell configurations.

## When to Use

- Displaying structured data in rows and columns
- Data grids with sorting, filtering, or selection
- Financial data, reports, and dashboards
- User lists, product catalogs, inventory views

## Anatomy

The Table consists of several sub-components:
- **Table**: Container wrapper with overflow handling
- **TableHead**: Header section with column labels
- **TableBody**: Main content area with data rows
- **TableFooter**: Footer section for totals or summaries
- **TableRow**: Individual row in the table
- **TableCell**: Individual cell, can be `td` or `th`

## Usage

### Basic Table

```tsx
import { 
  Table, TableHead, TableBody, TableRow, TableCell 
} from '@nest/components';

function BasicTable() {
  return (
    <Table>
      <TableHead>
        <TableRow>
          <TableCell component="th">Name</TableCell>
          <TableCell component="th">Email</TableCell>
          <TableCell component="th">Role</TableCell>
        </TableRow>
      </TableHead>
      <TableBody>
        <TableRow>
          <TableCell>John Doe</TableCell>
          <TableCell>john@example.com</TableCell>
          <TableCell>Admin</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>Jane Smith</TableCell>
          <TableCell>jane@example.com</TableCell>
          <TableCell>Editor</TableCell>
        </TableRow>
      </TableBody>
    </Table>
  );
}
```

### Numeric Columns

```tsx
<Table>
  <TableHead>
    <TableRow>
      <TableCell component="th">Product</TableCell>
      <TableCell component="th" numeric>Price</TableCell>
      <TableCell component="th" numeric>Stock</TableCell>
    </TableRow>
  </TableHead>
  <TableBody>
    <TableRow>
      <TableCell>Laptop Pro</TableCell>
      <TableCell numeric>$1,299.99</TableCell>
      <TableCell numeric>45</TableCell>
    </TableRow>
  </TableBody>
</Table>
```

### Sticky Header

For long tables that scroll:

```tsx
<div className="h-64 overflow-auto">
  <Table stickyHeader>
    <TableHead>
      <TableRow>
        <TableCell component="th">Header</TableCell>
      </TableRow>
    </TableHead>
    <TableBody>
      {/* Many rows */}
    </TableBody>
  </Table>
</div>
```

### Table with Footer

```tsx
<Table>
  <TableHead>
    <TableRow>
      <TableCell component="th">Item</TableCell>
      <TableCell component="th" numeric>Amount</TableCell>
    </TableRow>
  </TableHead>
  <TableBody>
    <TableRow>
      <TableCell>Subtotal</TableCell>
      <TableCell numeric>$100.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell>Tax</TableCell>
      <TableCell numeric>$10.00</TableCell>
    </TableRow>
  </TableBody>
  <TableFooter>
    <TableRow>
      <TableCell component="th">Total</TableCell>
      <TableCell numeric>$110.00</TableCell>
    </TableRow>
  </TableFooter>
</Table>
```

### Hover Rows

```tsx
<Table>
  <TableBody>
    <TableRow hover>
      <TableCell>Hoverable Row</TableCell>
    </TableRow>
  </TableBody>
</Table>
```

### Cell with ColSpan

```tsx
<TableRow>
  <TableCell>Name</TableCell>
  <TableCell colSpan={2}>
    Spans two columns
  </TableCell>
</TableRow>
```

## Props

### Table

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `fullWidth` | `boolean` | `true` | Table width is 100% |
| `stickyHeader` | `boolean` | `false` | Header sticks to top on scroll |
| `className` | `string` | - | Additional CSS classes |

### TableRow

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `border` | `boolean` | `true` | Show bottom border |
| `hover` | `boolean` | `false` | Enable hover effect |
| `className` | `string` | - | Additional CSS classes |

### TableCell

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `'td' \| 'th'` | `'td'` | HTML element to render |
| `numeric` | `boolean` | `false` | Right-align for numbers |
| `center` | `boolean` | `false` | Center-align content |
| `right` | `boolean` | `false` | Right-align content |
| `colSpan` | `number` | - | Column span |
| `rowSpan` | `number` | - | Row span |
| `className` | `string` | - | Additional CSS classes |

## Accessibility

- Use `component="th"` for header cells
- Use `scope` attribute on `th` cells for complex tables
- Provide meaningful row/column structure
- Consider adding `aria-sort` for sortable columns

## Best Practices

- **Use appropriate alignment**: Left for text, right for numbers, center for actions/icons
- **Keep headers concise**: Column headers should be short and descriptive
- **Consider responsive behavior**: Tables may need horizontal scroll on mobile
- **Use sticky headers for long lists**: Improves navigation in scrollable tables
- **Add hover states for interactable rows**: Visual feedback for clickable/selectable rows

## Related Components

- **List**: Alternative for simpler, non-tabular lists
- **Card**: Alternative for detailed item views
- **DataGrid**: Advanced table with sorting, filtering, pagination (coming soon)
