import { useState } from 'react';
import { List, ListItem, ListItemButton, ListItemText, ListItemIcon, ListItemAvatar } from './List';
import { Avatar } from './Avatar';
import { Inbox, Star, Send, Trash, Mail, Phone, User } from 'lucide-react';

/**
 * List Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function ListDemos() {
  const [selectedIndex, setSelectedIndex] = useState(0);

  return (
    <div className="space-y-8 p-6">
      {/* Basic List */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic List</h2>
        <List>
          <ListItem>Item 1</ListItem>
          <ListItem>Item 2</ListItem>
          <ListItem>Item 3</ListItem>
        </List>
      </section>

      {/* Dense List */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Dense List</h2>
        <List dense>
          <ListItem>Dense item 1</ListItem>
          <ListItem>Dense item 2</ListItem>
          <ListItem>Dense item 3</ListItem>
          <ListItem>Dense item 4</ListItem>
        </List>
        <p className="mt-2 text-sm text-nest-muted">Compact vertical padding for dense layouts</p>
      </section>

      {/* List with Icons */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">List with Icons</h2>
        <List>
          <ListItem>
            <ListItemIcon>
              <Inbox className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Inbox" secondary="12 messages" />
          </ListItem>
          <ListItem>
            <ListItemIcon>
              <Star className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Starred" secondary="5 items" />
          </ListItem>
          <ListItem>
            <ListItemIcon>
              <Send className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Sent" secondary="24 messages" />
          </ListItem>
          <ListItem>
            <ListItemIcon>
              <Trash className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Trash" secondary="3 items" />
          </ListItem>
        </List>
      </section>

      {/* List with Avatars */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">List with Avatars</h2>
        <List>
          <ListItem>
            <ListItemAvatar>
              <Avatar src="https://i.pravatar.cc/150?img=1" alt="User 1" />
            </ListItemAvatar>
            <ListItemText primary="John Doe" secondary="john@example.com" />
          </ListItem>
          <ListItem>
            <ListItemAvatar>
              <Avatar src="https://i.pravatar.cc/150?img=2" alt="User 2" />
            </ListItemAvatar>
            <ListItemText primary="Jane Smith" secondary="jane@example.com" />
          </ListItem>
          <ListItem>
            <ListItemAvatar>
              <Avatar src="https://i.pravatar.cc/150?img=3" alt="User 3" />
            </ListItemAvatar>
            <ListItemText primary="Bob Wilson" secondary="bob@example.com" />
          </ListItem>
        </List>
      </section>

      {/* Interactive List */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Interactive List</h2>
        <List>
          <ListItemButton
            selected={selectedIndex === 0}
            onClick={() => setSelectedIndex(0)}
          >
            <ListItemIcon>
              <Inbox className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Inbox" />
          </ListItemButton>
          <ListItemButton
            selected={selectedIndex === 1}
            onClick={() => setSelectedIndex(1)}
          >
            <ListItemIcon>
              <Star className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Starred" />
          </ListItemButton>
          <ListItemButton
            selected={selectedIndex === 2}
            onClick={() => setSelectedIndex(2)}
          >
            <ListItemIcon>
              <Send className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Sent" />
          </ListItemButton>
        </List>
        <p className="mt-2 text-sm text-nest-muted">Selected index: {selectedIndex}</p>
      </section>

      {/* Disabled List Items */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled List Items</h2>
        <List>
          <ListItemButton>
            <ListItemIcon>
              <Mail className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Enabled Item" />
          </ListItemButton>
          <ListItemButton disabled>
            <ListItemIcon>
              <Mail className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Disabled Item" />
          </ListItemButton>
          <ListItemButton>
            <ListItemIcon>
              <Mail className="h-5 w-5" />
            </ListItemIcon>
            <ListItemText primary="Enabled Item" />
          </ListItemButton>
        </List>
      </section>

      {/* Contact List */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Contact List</h2>
        <div className="max-w-md border border-nest-border rounded-nest-md">
          <List>
            <ListItem>
              <ListItemAvatar>
                <Avatar>JD</Avatar>
              </ListItemAvatar>
              <ListItemText
                primary="John Doe"
                secondary={
                  <div className="flex items-center gap-1">
                    <Phone className="h-3 w-3" />
                    <span>+1 (555) 123-4567</span>
                  </div>
                }
              />
            </ListItem>
            <ListItem>
              <ListItemAvatar>
                <Avatar>JS</Avatar>
              </ListItemAvatar>
              <ListItemText
                primary="Jane Smith"
                secondary={
                  <div className="flex items-center gap-1">
                    <Phone className="h-3 w-3" />
                    <span>+1 (555) 987-6543</span>
                  </div>
                }
              />
            </ListItem>
            <ListItem>
              <ListItemAvatar>
                <Avatar>BW</Avatar>
              </ListItemAvatar>
              <ListItemText
                primary="Bob Wilson"
                secondary={
                  <div className="flex items-center gap-1">
                    <Phone className="h-3 w-3" />
                    <span>+1 (555) 456-7890</span>
                  </div>
                }
              />
            </ListItem>
          </List>
        </div>
      </section>

      {/* Settings List */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Settings List</h2>
        <div className="max-w-md border border-nest-border rounded-nest-md">
          <List>
            <ListItemButton>
              <ListItemIcon>
                <User className="h-5 w-5" />
              </ListItemIcon>
              <ListItemText primary="Account" secondary="Manage your account" />
            </ListItemButton>
            <ListItemButton>
              <ListItemIcon>
                <Mail className="h-5 w-5" />
              </ListItemIcon>
              <ListItemText primary="Notifications" secondary="Email preferences" />
            </ListItemButton>
            <ListItemButton>
              <ListItemIcon>
                <Star className="h-5 w-5" />
              </ListItemIcon>
              <ListItemText primary="Appearance" secondary="Theme and display" />
            </ListItemButton>
          </List>
        </div>
      </section>

      {/* List with Only Primary Text */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Primary Text Only</h2>
        <List>
          <ListItem>
            <ListItemText primary="Single line item" />
          </ListItem>
          <ListItem>
            <ListItemText primary="Another single line" />
          </ListItem>
          <ListItem>
            <ListItemText primary="Yet another item" />
          </ListItem>
        </List>
      </section>

      {/* List with Only Secondary Text */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Secondary Text Only</h2>
        <List>
          <ListItem>
            <ListItemText secondary="Secondary text only" />
          </ListItem>
          <ListItem>
            <ListItemText secondary="Another secondary text" />
          </ListItem>
          <ListItem>
            <ListItemText secondary="Yet another secondary" />
          </ListItem>
        </List>
      </section>

      {/* Long Text Truncation */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Long Text Truncation</h2>
        <div className="max-w-xs border border-nest-border rounded-nest-md">
          <List>
            <ListItem>
              <ListItemText
                primary="This is a very long primary text that should truncate with ellipsis when it overflows"
                secondary="This is also a very long secondary text that should truncate when it overflows the container width"
              />
            </ListItem>
            <ListItem>
              <ListItemText
                primary="Short text"
                secondary="Short secondary"
              />
            </ListItem>
          </List>
        </div>
      </section>
    </div>
  );
}
