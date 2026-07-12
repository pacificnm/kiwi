import { useState } from 'react';
import { Modal } from './Modal';
import { Button } from '../inputs/Button';
import { Stack } from '../layout/Stack';
import { Typography } from '../data-display/Typography';

export default function ModalDemo() {
  const [openBasic, setOpenBasic] = useState(false);
  const [openNoEscape, setOpenNoEscape] = useState(false);

  return (
    <div className="p-4 space-y-4">
      <Typography variant="h5">Modal</Typography>
      <Typography variant="body2" className="text-nest-muted">
        A dialog overlay with focus trap and dismissible behavior.
      </Typography>

      <Stack direction="row" className="flex-wrap gap-2">
        <Button variant="contained" onClick={() => setOpenBasic(true)}>
          Open Basic Modal
        </Button>
        <Button variant="outlined" onClick={() => setOpenNoEscape(true)}>
          No Escape Close
        </Button>
      </Stack>

      {/* Basic Modal */}
      <Modal open={openBasic} onClose={() => setOpenBasic(false)}>
        <div className="p-6 min-w-[300px] max-w-[500px]">
          <Typography variant="h6" className="mb-2">
            Basic Modal
          </Typography>
          <Typography variant="body2" className="text-nest-muted mb-4">
            This is a basic modal dialog. Click outside or press Escape to close.
          </Typography>
          <Stack direction="row" className="justify-end gap-2">
            <Button variant="outlined" onClick={() => setOpenBasic(false)}>
              Cancel
            </Button>
            <Button variant="contained" onClick={() => setOpenBasic(false)}>
              Confirm
            </Button>
          </Stack>
        </div>
      </Modal>

      {/* Modal without Escape close */}
      <Modal
        open={openNoEscape}
        onClose={() => setOpenNoEscape(false)}
        closeOnEscape={false}
      >
        <div className="p-6 min-w-[300px]">
          <Typography variant="h6" className="mb-2">
            No Escape Close
          </Typography>
          <Typography variant="body2" className="text-nest-muted mb-4">
            This modal can only be closed by clicking the button or outside.
            Pressing Escape won't close it.
          </Typography>
          <Button variant="contained" onClick={() => setOpenNoEscape(false)}>
            Close
          </Button>
        </div>
      </Modal>
    </div>
  );
}
