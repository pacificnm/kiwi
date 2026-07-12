import { useState } from 'react';
import { Alert } from './Alert';
import { Button } from '../inputs/Button';

/**
 * Alert Component Demos
 */

export function AlertDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Severity Levels */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Severity Levels</h2>
        <div className="space-y-4">
          <Alert severity="success">
            <strong>Success!</strong> Your changes have been saved.
          </Alert>
          <Alert severity="error">
            <strong>Error!</strong> Something went wrong. Please try again.
          </Alert>
          <Alert severity="warning">
            <strong>Warning!</strong> Your session is about to expire.
          </Alert>
          <Alert severity="info">
            <strong>Info:</strong> A new version is available.
          </Alert>
        </div>
      </section>

      {/* Variants */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variants</h2>
        <div className="space-y-4">
          <Alert severity="info" variant="filled">Filled variant</Alert>
          <Alert severity="info" variant="outlined">Outlined variant</Alert>
          <Alert severity="info" variant="standard">Standard variant</Alert>
        </div>
      </section>

      {/* With Close Button */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Close Button</h2>
        <AlertWithClose />
      </section>

      {/* With Action */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Action</h2>
        <AlertWithAction />
      </section>

      {/* Without Icon */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Without Icon</h2>
        <Alert severity="success" icon={null}>
          This alert has no icon.
        </Alert>
      </section>
    </div>
  );
}

function AlertWithClose() {
  const [open, setOpen] = useState(true);

  if (!open) {
    return (
      <Button variant="outlined" onClick={() => setOpen(true)}>
        Show Alert
      </Button>
    );
  }

  return (
    <Alert
      severity="warning"
      onClose={() => setOpen(false)}
    >
      This is a dismissible alert. Click the X to close it.
    </Alert>
  );
}

function AlertWithAction() {
  const [open, setOpen] = useState(true);

  if (!open) {
    return (
      <Button variant="outlined" onClick={() => setOpen(true)}>
        Show Alert
      </Button>
    );
  }

  return (
    <Alert
      severity="error"
      onClose={() => setOpen(false)}
      action={
        <Button size="small" color="error" variant="outlined" onClick={() => alert('Retry clicked!')}>
          Retry
        </Button>
      }
    >
      <strong>Connection failed.</strong> Unable to reach the server.
    </Alert>
  );
}
