import { useState } from 'react';
import { Snackbar } from './Snackbar';
import { Button } from '../inputs/Button';

/**
 * Snackbar Component Demos
 */

export function SnackbarDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Basic Snackbar */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Snackbar</h2>
        <BasicSnackbarDemo />
      </section>

      {/* Severity Levels */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Severity Levels</h2>
        <SeverityDemo />
      </section>

      {/* With Action */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Action</h2>
        <ActionSnackbarDemo />
      </section>

      {/* Positions */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Positions</h2>
        <PositionDemo />
      </section>
    </div>
  );
}

function BasicSnackbarDemo() {
  const [open, setOpen] = useState(false);

  return (
    <div className="rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <Button variant="contained" onClick={() => setOpen(true)}>
        Show Snackbar
      </Button>
      <Snackbar
        open={open}
        onClose={() => setOpen(false)}
        autoHideDuration={3000}
      >
        Message saved successfully!
      </Snackbar>
    </div>
  );
}

function SeverityDemo() {
  const [severity, setSeverity] = useState<'success' | 'error' | 'warning' | 'info'>('info');
  const [open, setOpen] = useState(false);

  return (
    <div className="rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <div className="flex flex-wrap gap-2">
        <Button size="small" onClick={() => { setSeverity('success'); setOpen(true); }}>
          Success
        </Button>
        <Button size="small" color="error" onClick={() => { setSeverity('error'); setOpen(true); }}>
          Error
        </Button>
        <Button size="small" color="warning" onClick={() => { setSeverity('warning'); setOpen(true); }}>
          Warning
        </Button>
        <Button size="small" color="secondary" onClick={() => { setSeverity('info'); setOpen(true); }}>
          Info
        </Button>
      </div>
      <Snackbar
        open={open}
        onClose={() => setOpen(false)}
        severity={severity}
      >
        This is a {severity} message.
      </Snackbar>
    </div>
  );
}

function ActionSnackbarDemo() {
  const [open, setOpen] = useState(false);
  const [actionTaken, setActionTaken] = useState('');

  return (
    <div className="rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <Button variant="contained" onClick={() => setOpen(true)}>
        Delete Item
      </Button>
      {actionTaken && (
        <p className="mt-2 text-sm text-nest-muted">{actionTaken}</p>
      )}
      <Snackbar
        open={open}
        onClose={() => setOpen(false)}
        severity="success"
        action={
          <Button
            size="small"
            variant="outlined"
            onClick={() => {
              setActionTaken('Undo performed!');
              setOpen(false);
              setTimeout(() => setActionTaken(''), 2000);
            }}
          >
            Undo
          </Button>
        }
      >
        Item deleted
      </Snackbar>
    </div>
  );
}

function PositionDemo() {
  const [position, setPosition] = useState<SnackbarProps['position']>('bottom-center');
  const [open, setOpen] = useState(false);

  const positions: SnackbarProps['position'][] = [
    'top-left', 'top-center', 'top-right',
    'bottom-left', 'bottom-center', 'bottom-right'
  ];

  return (
    <div className="rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <div className="flex flex-wrap gap-2">
        {positions.map((pos) => (
          <Button
            key={pos}
            size="small"
            variant={position === pos ? 'contained' : 'outlined'}
            onClick={() => {
              setPosition(pos);
              setOpen(true);
            }}
          >
            {pos}
          </Button>
        ))}
      </div>
      <Snackbar
        open={open}
        onClose={() => setOpen(false)}
        position={position}
      >
        Position: {position}
      </Snackbar>
    </div>
  );
}

// Import for type reference
import type { SnackbarProps } from './Snackbar';
