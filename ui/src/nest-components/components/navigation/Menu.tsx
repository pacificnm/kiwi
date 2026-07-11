import {
  createContext,
  forwardRef,
  useContext,
  useEffect,
  useRef,
  useState,
  type ButtonHTMLAttributes,
  type HTMLAttributes,
  type ReactNode,
} from 'react';
import { cn } from '../../lib/cn';

export interface MenuProps extends Omit<HTMLAttributes<HTMLDivElement>, 'onClose'> {
  /** Whether the dropdown is visible. */
  open: boolean;
  /**
   * Called on outside click or Escape. Not called on item selection —
   * call it yourself from each {@link MenuItem}'s `onClick`, same as
   * closing a native dropdown (`onClick={() => { doThing(); onClose(); }}`).
   */
  onClose: () => void;
  children?: ReactNode;
}

/**
 * Dropdown menu panel, positioned absolutely under its nearest
 * `position: relative` ancestor (typically the trigger button's wrapper).
 * Closes on outside click / Escape. Pair with {@link MenuItem} and
 * {@link MenuDivider}; use {@link MenuBar} + {@link MenuBarItem} for a
 * File/Edit-style top menu row.
 *
 * @example
 * <div className="relative inline-block">
 *   <Button onClick={() => setOpen(true)}>Options</Button>
 *   <Menu open={open} onClose={() => setOpen(false)}>
 *     <MenuItem onClick={() => { save(); setOpen(false); }}>Save</MenuItem>
 *     <MenuDivider />
 *     <MenuItem danger onClick={() => { remove(); setOpen(false); }}>Delete</MenuItem>
 *   </Menu>
 * </div>
 */
export const Menu = forwardRef<HTMLDivElement, MenuProps>(function Menu(
  { open, onClose, className, children, ...props },
  ref
) {
  const localRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    const onPointerDown = (event: MouseEvent) => {
      const target = event.target;
      if (target instanceof Node && localRef.current?.parentElement?.contains(target)) {
        return;
      }
      onClose();
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    window.addEventListener('mousedown', onPointerDown, true);
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('mousedown', onPointerDown, true);
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div
      ref={(node) => {
        localRef.current = node;
        if (typeof ref === 'function') {
          ref(node);
        } else if (ref) {
          ref.current = node;
        }
      }}
      role="menu"
      className={cn(
        'absolute left-0 top-full z-50 min-w-48 rounded-nest-md border border-nest-border bg-nest-surface py-1 shadow-lg',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
});

export interface MenuItemProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Renders in the error color, for destructive actions (e.g. Delete). */
  danger?: boolean;
  /** Trailing content — a keyboard shortcut, chevron, or badge. */
  endAdornment?: ReactNode;
  children?: ReactNode;
}

/** One row inside a {@link Menu}. */
export const MenuItem = forwardRef<HTMLButtonElement, MenuItemProps>(function MenuItem(
  { className, danger, disabled, endAdornment, children, ...props },
  ref
) {
  return (
    <button
      ref={ref}
      type="button"
      role="menuitem"
      disabled={disabled}
      className={cn(
        'flex w-full items-center justify-between gap-6 px-3 py-1.5 text-left text-sm transition-colors',
        disabled
          ? 'cursor-default text-nest-muted/50'
          : danger
            ? 'text-nest-error hover:bg-nest-error/10'
            : 'text-nest-foreground hover:bg-nest-muted/10',
        className
      )}
      {...props}
    >
      <span className="flex items-center gap-1.5">{children}</span>
      {endAdornment ? <span className="text-xs text-nest-muted">{endAdornment}</span> : null}
    </button>
  );
});

/** Horizontal divider between {@link MenuItem} groups. */
export function MenuDivider({ className }: { className?: string }) {
  return <div role="separator" className={cn('my-1 h-px bg-nest-border', className)} />;
}

type MenuBarContextValue = {
  openId: string | null;
  setOpenId: (id: string | null) => void;
};

const MenuBarContext = createContext<MenuBarContextValue | null>(null);

export interface MenuBarProps extends HTMLAttributes<HTMLDivElement> {
  children?: ReactNode;
}

/**
 * Horizontal row of top-level menu triggers (File / Edit / Help), like a
 * desktop app's menu bar. Only one {@link MenuBarItem} dropdown is open at a
 * time; clicking outside the bar or pressing Escape closes it.
 *
 * @example
 * <MenuBar>
 *   <MenuBarItem id="file" label="File">
 *     <MenuItem onClick={() => openFile()}>Open…</MenuItem>
 *   </MenuBarItem>
 *   <MenuBarItem id="edit" label="Edit">
 *     <MenuItem onClick={() => undo()}>Undo</MenuItem>
 *   </MenuBarItem>
 * </MenuBar>
 */
export function MenuBar({ className, children, ...props }: MenuBarProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [openId, setOpenId] = useState<string | null>(null);

  useEffect(() => {
    if (!openId) {
      return;
    }
    const onPointerDown = (event: MouseEvent) => {
      if (event.target instanceof Node && ref.current?.contains(event.target)) {
        return;
      }
      setOpenId(null);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setOpenId(null);
      }
    };
    window.addEventListener('mousedown', onPointerDown, true);
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('mousedown', onPointerDown, true);
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [openId]);

  return (
    <MenuBarContext.Provider value={{ openId, setOpenId }}>
      <div
        ref={ref}
        role="menubar"
        className={cn('flex h-full items-stretch', className)}
        {...props}
      >
        {children}
      </div>
    </MenuBarContext.Provider>
  );
}

export interface MenuBarItemProps {
  /** Stable id — must be unique within the enclosing {@link MenuBar}. */
  id: string;
  label: string;
  disabled?: boolean;
  children?: ReactNode;
}

/** One top-level menu ("File", "Edit", …) with its dropdown, inside a {@link MenuBar}. */
export function MenuBarItem({ id, label, disabled, children }: MenuBarItemProps) {
  const ctx = useContext(MenuBarContext);
  if (!ctx) {
    throw new Error('MenuBarItem must be used within a MenuBar');
  }
  const open = ctx.openId === id;

  return (
    <div className="relative flex h-full items-stretch">
      <button
        type="button"
        role="menuitem"
        aria-haspopup="true"
        aria-expanded={open}
        disabled={disabled}
        onClick={() => ctx.setOpenId(open ? null : id)}
        className={cn(
          'h-full px-2.5 text-sm transition-colors',
          disabled
            ? 'cursor-default text-nest-muted/50'
            : open
              ? 'bg-nest-muted/15 text-nest-foreground'
              : 'text-nest-foreground hover:bg-nest-muted/10'
        )}
      >
        {label}
      </button>
      <Menu open={open} onClose={() => ctx.setOpenId(null)}>
        {children}
      </Menu>
    </div>
  );
}
