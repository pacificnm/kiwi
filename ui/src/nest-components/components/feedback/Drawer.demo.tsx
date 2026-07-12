import { useState } from 'react';
import { Drawer } from './Drawer';
import { Button } from '../inputs/Button';
import { Stack } from '../layout/Stack';
import { Typography } from '../data-display/Typography';
import { List, ListItem, ListItemText } from '../data-display/List';
import { Divider } from '../layout/Divider';

export default function DrawerDemo() {
  const [openLeft, setOpenLeft] = useState(false);
  const [openRight, setOpenRight] = useState(false);
  const [openTop, setOpenTop] = useState(false);
  const [openBottom, setOpenBottom] = useState(false);

  const navItems = ['Dashboard', 'Projects', 'Tasks', 'Calendar', 'Settings'];

  return (
    <div className="p-4 space-y-4">
      <Typography variant="h5">Drawer</Typography>
      <Typography variant="body2" className="text-nest-muted">
        A slide-out panel from the edge of the screen.
      </Typography>

      <Stack direction="row" className="flex-wrap gap-2">
        <Button variant="outlined" onClick={() => setOpenLeft(true)}>
          Left Drawer
        </Button>
        <Button variant="outlined" onClick={() => setOpenRight(true)}>
          Right Drawer
        </Button>
        <Button variant="outlined" onClick={() => setOpenTop(true)}>
          Top Drawer
        </Button>
        <Button variant="outlined" onClick={() => setOpenBottom(true)}>
          Bottom Drawer
        </Button>
      </Stack>

      {/* Left Navigation Drawer */}
      <Drawer open={openLeft} onClose={() => setOpenLeft(false)} anchor="left" width={280}>
        <div className="p-4">
          <Typography variant="h6" className="mb-4">
            Navigation
          </Typography>
          <List>
            {navItems.map((item) => (
              <ListItem key={item}>
                <ListItemText primary={item} />
              </ListItem>
            ))}
          </List>
        </div>
      </Drawer>

      {/* Right Settings Drawer */}
      <Drawer open={openRight} onClose={() => setOpenRight(false)} anchor="right" width={320}>
        <div className="p-4">
          <Typography variant="h6" className="mb-2">
            Settings
          </Typography>
          <Typography variant="body2" className="text-nest-muted mb-4">
            Configure your preferences
          </Typography>
          <Divider className="my-4" />
          <Stack direction="column" className="space-y-4">
            <div>
              <Typography variant="subtitle2">Theme</Typography>
              <Stack direction="row" className="gap-2 mt-2">
                <Button size="small" variant="outlined">Light</Button>
                <Button size="small" variant="contained">Dark</Button>
                <Button size="small" variant="outlined">System</Button>
              </Stack>
            </div>
          </Stack>
        </div>
      </Drawer>

      {/* Top Notification Drawer */}
      <Drawer open={openTop} onClose={() => setOpenTop(false)} anchor="top" height={200}>
        <div className="p-4">
          <Typography variant="h6" className="mb-2">
            Notifications
          </Typography>
          <Typography variant="body2" className="text-nest-muted">
            You have 3 new notifications
          </Typography>
          <List className="mt-4">
            <ListItem>
              <ListItemText primary="New comment on your post" secondary="2 minutes ago" />
            </ListItem>
            <ListItem>
              <ListItemText primary="Task completed" secondary="1 hour ago" />
            </ListItem>
          </List>
        </div>
      </Drawer>

      {/* Bottom Player Drawer */}
      <Drawer open={openBottom} onClose={() => setOpenBottom(false)} anchor="bottom" height={120}>
        <div className="p-4 flex items-center justify-between">
          <div>
            <Typography variant="subtitle1">Now Playing</Typography>
            <Typography variant="body2" className="text-nest-muted">
              Example Song - Artist
            </Typography>
          </div>
          <Stack direction="row" className="gap-2">
            <Button variant="outlined" size="small">Previous</Button>
            <Button variant="contained" size="small">Pause</Button>
            <Button variant="outlined" size="small">Next</Button>
          </Stack>
        </div>
      </Drawer>
    </div>
  );
}
