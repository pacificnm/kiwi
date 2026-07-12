import { forwardRef, type ReactNode, type HTMLAttributes } from 'react';
import { cn } from '../../../lib/cn';

// Table Props
export interface TableProps {
  /**
   * The content of the table.
   */
  children?: ReactNode;
  /**
   * If true, the table width is 100%.
   * @default true
   */
  fullWidth?: boolean;
  /**
   * If true, the table is sticky.
   * @default false
   */
  stickyHeader?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Table component - container for tabular data.
 * Follows MUI Table API conventions.
 *
 * @example
 * // Basic table
 * <Table>
 *   <TableHead>
 *     <TableRow>
 *       <TableCell>Header 1</TableCell>
 *       <TableCell>Header 2</TableCell>
 *     </TableRow>
 *   </TableHead>
 *   <TableBody>
 *     <TableRow>
 *       <TableCell>Cell 1</TableCell>
 *       <TableCell>Cell 2</TableCell>
 *     </TableRow>
 *   </TableBody>
 * </Table>
 */
export const Table = forwardRef<HTMLTableElement, TableProps>(function Table(
  {
    fullWidth = true,
    stickyHeader = false,
    className,
    children,
    ...props
  }: TableProps,
  ref: React.Ref<HTMLTableElement>
) {
  return (
    <div className={cn('overflow-auto', fullWidth && 'w-full')}>
      <table
        ref={ref}
        className={cn(
          'w-full',
          'text-left',
          'border-collapse',
          stickyHeader && '[&_thead]:sticky [&_thead]:top-0 [&_thead]:bg-nest-surface [&_thead]:z-10',
          className
        )}
        {...props}
      >
        {children}
      </table>
    </div>
  );
});

// TableHead Props
export interface TableHeadProps extends HTMLAttributes<HTMLTableSectionElement> {
  /**
   * The content of the table head.
   */
  children?: ReactNode;
}

/**
 * TableHead component - wraps the header rows of a table.
 */
export const TableHead = forwardRef<HTMLTableSectionElement, TableHeadProps>(function TableHead(
  { className, children, ...props }: TableHeadProps,
  ref: React.Ref<HTMLTableSectionElement>
) {
  return (
    <thead ref={ref} className={cn('bg-nest-surface', className)} {...props}>
      {children}
    </thead>
  );
});

// TableBody Props
export interface TableBodyProps extends HTMLAttributes<HTMLTableSectionElement> {
  /**
   * The content of the table body.
   */
  children?: ReactNode;
}

/**
 * TableBody component - wraps the main content rows of a table.
 */
export const TableBody = forwardRef<HTMLTableSectionElement, TableBodyProps>(function TableBody(
  { className, children, ...props }: TableBodyProps,
  ref: React.Ref<HTMLTableSectionElement>
) {
  return (
    <tbody ref={ref} className={cn(className)} {...props}>
      {children}
    </tbody>
  );
});

// TableFooter Props
export interface TableFooterProps extends HTMLAttributes<HTMLTableSectionElement> {
  /**
   * The content of the table footer.
   */
  children?: ReactNode;
}

/**
 * TableFooter component - wraps the footer rows of a table.
 */
export const TableFooter = forwardRef<HTMLTableSectionElement, TableFooterProps>(function TableFooter(
  { className, children, ...props }: TableFooterProps,
  ref: React.Ref<HTMLTableSectionElement>
) {
  return (
    <tfoot
      ref={ref}
      className={cn('bg-nest-surface font-medium', className)}
      {...props}
    >
      {children}
    </tfoot>
  );
});

// TableRow Props
export interface TableRowProps extends HTMLAttributes<HTMLTableRowElement> {
  /**
   * If true, the row has a bottom border.
   * @default true
   */
  border?: boolean;
  /**
   * If true, the row has hover effect.
   * @default false
   */
  hover?: boolean;
  /**
   * The content of the row.
   */
  children?: ReactNode;
}

/**
 * TableRow component - a single row in a table.
 */
export const TableRow = forwardRef<HTMLTableRowElement, TableRowProps>(function TableRow(
  {
    border = true,
    hover = false,
    className,
    children,
    ...props
  }: TableRowProps,
  ref: React.Ref<HTMLTableRowElement>
) {
  return (
    <tr
      ref={ref}
      className={cn(
        border && 'border-b border-nest-border',
        hover && 'hover:bg-nest-surface transition-colors',
        className
      )}
      {...props}
    >
      {children}
    </tr>
  );
});

// TableCell Props
export interface TableCellProps extends HTMLAttributes<HTMLTableCellElement> {
  /**
   * The component to render as.
   * @default 'td'
   */
  component?: 'td' | 'th';
  /**
   * If true, the cell has numeric alignment.
   * @default false
   */
  numeric?: boolean;
  /**
   * If true, the cell has center alignment.
   * @default false
   */
  center?: boolean;
  /**
   * If true, the cell has right alignment.
   * @default false
   */
  right?: boolean;
  /**
   * Column span.
   */
  colSpan?: number;
  /**
   * Row span.
   */
  rowSpan?: number;
  /**
   * The content of the cell.
   */
  children?: ReactNode;
}

/**
 * TableCell component - a single cell in a table row.
 */
export const TableCell = forwardRef<HTMLTableCellElement, TableCellProps>(function TableCell(
  {
    component: Component = 'td',
    numeric = false,
    center = false,
    right = false,
    colSpan,
    rowSpan,
    className,
    children,
    ...props
  }: TableCellProps,
  ref: React.Ref<HTMLTableCellElement>
) {
  const alignClass = numeric || right ? 'text-right' : center ? 'text-center' : 'text-left';

  return (
    <Component
      ref={ref}
      colSpan={colSpan}
      rowSpan={rowSpan}
      className={cn(
        'px-4 py-3',
        'text-sm',
        alignClass,
        Component === 'th' && 'font-medium text-nest-foreground',
        className
      )}
      {...props}
    >
      {children}
    </Component>
  );
});
