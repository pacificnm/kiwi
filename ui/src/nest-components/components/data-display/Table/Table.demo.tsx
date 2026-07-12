import { Table, TableHead, TableBody, TableRow, TableCell, TableFooter } from './Table';
import { Typography } from '../Typography';
import { Chip } from '../Chip';

const users = [
  { id: 1, name: 'John Doe', email: 'john@example.com', role: 'Admin', status: 'Active' },
  { id: 2, name: 'Jane Smith', email: 'jane@example.com', role: 'Editor', status: 'Active' },
  { id: 3, name: 'Bob Johnson', email: 'bob@example.com', role: 'Viewer', status: 'Inactive' },
  { id: 4, name: 'Alice Brown', email: 'alice@example.com', role: 'Editor', status: 'Active' },
  { id: 5, name: 'Charlie Wilson', email: 'charlie@example.com', role: 'Viewer', status: 'Pending' },
];

const products = [
  { id: 1, name: 'Laptop Pro', category: 'Electronics', price: 1299.99, stock: 45 },
  { id: 2, name: 'Wireless Mouse', category: 'Electronics', price: 49.99, stock: 120 },
  { id: 3, name: 'Desk Chair', category: 'Furniture', price: 299.99, stock: 23 },
  { id: 4, name: 'Monitor 27"', category: 'Electronics', price: 399.99, stock: 67 },
];

export default function TableDemo() {
  return (
    <div className="p-4 space-y-8">
      <div>
        <Typography variant="h5" className="mb-4">Table</Typography>
        <Typography variant="body2" className="text-nest-muted mb-4">
          Components for displaying tabular data with sorting, selection, and pagination support.
        </Typography>
      </div>

      {/* Basic Table */}
      <div>
        <Typography variant="h6" className="mb-2">Basic Table</Typography>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell component="th">Name</TableCell>
              <TableCell component="th">Email</TableCell>
              <TableCell component="th">Role</TableCell>
              <TableCell component="th">Status</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {users.map((user) => (
              <TableRow key={user.id} hover>
                <TableCell>{user.name}</TableCell>
                <TableCell>{user.email}</TableCell>
                <TableCell>{user.role}</TableCell>
                <TableCell>
                  <Chip
                    label={user.status}
                    color={user.status === 'Active' ? 'success' : user.status === 'Inactive' ? 'default' : 'warning'}
                    size="small"
                  />
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      {/* Numeric Data Table */}
      <div>
        <Typography variant="h6" className="mb-2">Numeric Columns</Typography>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell component="th">Product</TableCell>
              <TableCell component="th">Category</TableCell>
              <TableCell component="th" numeric>Price</TableCell>
              <TableCell component="th" numeric>Stock</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {products.map((product) => (
              <TableRow key={product.id} hover>
                <TableCell>{product.name}</TableCell>
                <TableCell>{product.category}</TableCell>
                <TableCell numeric>${product.price.toFixed(2)}</TableCell>
                <TableCell numeric>{product.stock}</TableCell>
              </TableRow>
            ))}
          </TableBody>
          <TableFooter>
            <TableRow>
              <TableCell component="th">Total</TableCell>
              <TableCell></TableCell>
              <TableCell numeric>
                ${products.reduce((sum, p) => sum + p.price, 0).toFixed(2)}
              </TableCell>
              <TableCell numeric>
                {products.reduce((sum, p) => sum + p.stock, 0)}
              </TableCell>
            </TableRow>
          </TableFooter>
        </Table>
      </div>

      {/* Sticky Header Table */}
      <div>
        <Typography variant="h6" className="mb-2">Sticky Header</Typography>
        <div className="h-48 overflow-auto border border-nest-border rounded">
          <Table stickyHeader>
            <TableHead>
              <TableRow>
                <TableCell component="th">ID</TableCell>
                <TableCell component="th">Name</TableCell>
                <TableCell component="th">Email</TableCell>
                <TableCell component="th">Role</TableCell>
                <TableCell component="th">Status</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {[...users, ...users, ...users].map((user, index) => (
                <TableRow key={`${user.id}-${index}`} hover>
                  <TableCell>{user.id}</TableCell>
                  <TableCell>{user.name}</TableCell>
                  <TableCell>{user.email}</TableCell>
                  <TableCell>{user.role}</TableCell>
                  <TableCell>
                    <Chip
                      label={user.status}
                      color={user.status === 'Active' ? 'success' : user.status === 'Inactive' ? 'default' : 'warning'}
                      size="small"
                    />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>

      {/* ColSpan Example */}
      <div>
        <Typography variant="h6" className="mb-2">ColSpan</Typography>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell component="th">Name</TableCell>
              <TableCell component="th">Contact</TableCell>
              <TableCell component="th">Role</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {users.slice(0, 2).map((user) => (
              <TableRow key={user.id} hover>
                <TableCell>{user.name}</TableCell>
                <TableCell colSpan={2}>
                  <div className="text-sm">
                    <div>{user.email}</div>
                    <div className="text-nest-muted">{user.role}</div>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
