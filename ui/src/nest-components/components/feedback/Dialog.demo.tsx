import React, { useState } from 'react';
import { Dialog } from './Dialog';
import { Button } from '../inputs/Button';

/**
 * Dialog Component Demos
 */

export function DialogDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Basic Dialog */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Dialog</h2>
        <BasicDialogDemo />
      </section>

      {/* Confirmation Dialog */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Confirmation Dialog</h2>
        <ConfirmDialogDemo />
      </section>

      {/* Form Dialog */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Form Dialog</h2>
        <FormDialogDemo />
      </section>
    </div>
  );
}

function BasicDialogDemo() {
  const [open, setOpen] = useState(false);

  return (
    <div className="rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <Button variant="contained" onClick={() => setOpen(true)}>
        Open Dialog
      </Button>
      <p className="mt-2 text-sm text-nest-muted">
        A simple dialog with title and close button
      </p>

      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        title="Welcome to Nest Components"
      >
        <p className="text-nest-foreground">
          This is a basic dialog demonstration. You can put any content here,
          including text, images, forms, or other components.
        </p>
      </Dialog>
    </div>
  );
}

function ConfirmDialogDemo() {
  const [open, setOpen] = useState(false);
  const [result, setResult] = useState<string>('');

  const handleConfirm = () => {
    setResult('Item deleted!');
    setOpen(false);
    setTimeout(() => setResult(''), 3000);
  };

  return (
    <div className="rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <Button variant="contained" color="error" onClick={() => setOpen(true)}>
        Delete Item
      </Button>
      {result && (
        <p className="mt-2 text-sm text-nest-success">{result}</p>
      )}

      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        title="Delete Item?"
        actions={
          <>
            <Button variant="text" onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button variant="contained" color="error" onClick={handleConfirm}>
              Delete
            </Button>
          </>
        }
      >
        <p className="text-nest-foreground">
          Are you sure you want to delete this item? This action cannot be undone.
        </p>
      </Dialog>
    </div>
  );
}

function FormDialogDemo() {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    alert(`Created: ${name}`);
    setName('');
    setOpen(false);
  };

  return (
    <div className="rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <Button variant="contained" onClick={() => setOpen(true)}>
        Create New
      </Button>

      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        title="Create New Item"
        actions={
          <>
            <Button variant="text" onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button variant="contained" onClick={handleSubmit}>
              Create
            </Button>
          </>
        }
      >
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="mb-1 block text-sm font-medium text-nest-foreground">
              Item Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-nest-md border border-nest-border bg-nest-background px-3 py-2 text-sm text-nest-foreground outline-none focus:border-nest-primary focus:ring-1 focus:ring-nest-primary"
              placeholder="Enter item name"
              autoFocus
            />
          </div>
        </form>
      </Dialog>
    </div>
  );
}
