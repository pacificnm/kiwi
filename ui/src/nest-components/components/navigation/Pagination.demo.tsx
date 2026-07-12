import { useState } from 'react';
import { Pagination } from './Pagination';

export function PaginationDemos() {
  const [page1, setPage1] = useState(1);
  const [page3, setPage3] = useState(1);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <div className="space-y-4">
          <Pagination count={10} page={page1} onChange={(_, p) => setPage1(p)} />
          <p className="text-sm text-nest-muted">Current page: {page1}</p>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Uncontrolled</h2>
        <Pagination count={10} defaultPage={1} />
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Many Pages</h2>
        <Pagination count={50} defaultPage={25} siblingCount={2} boundaryCount={2} />
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Small</p>
            <Pagination count={10} size="small" defaultPage={1} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Medium (default)</p>
            <Pagination count={10} size="medium" defaultPage={1} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Large</p>
            <Pagination count={10} size="large" defaultPage={1} />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Primary (default)</p>
            <Pagination count={10} color="primary" defaultPage={1} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Secondary</p>
            <Pagination count={10} color="secondary" defaultPage={1} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Success</p>
            <Pagination count={10} color="success" defaultPage={1} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Error</p>
            <Pagination count={10} color="error" defaultPage={1} />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Hidden Controls</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Hide first/last buttons</p>
            <Pagination count={10} hideFirstLast defaultPage={5} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Hide prev/next buttons</p>
            <Pagination count={10} hidePrevNext defaultPage={5} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Hide all navigation</p>
            <Pagination count={10} hideFirstLast hidePrevNext defaultPage={5} />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <Pagination count={10} disabled defaultPage={5} />
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Edge Cases</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Single page</p>
            <Pagination count={1} defaultPage={1} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Two pages</p>
            <Pagination count={2} defaultPage={1} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">First page</p>
            <Pagination count={10} defaultPage={1} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Last page</p>
            <Pagination count={10} defaultPage={10} />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Table</h2>
        <div className="border border-nest-border rounded-nest-md overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-nest-muted/50">
              <tr>
                <th className="px-4 py-2 text-left font-medium">Name</th>
                <th className="px-4 py-2 text-left font-medium">Email</th>
                <th className="px-4 py-2 text-left font-medium">Role</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-nest-border">
              <tr>
                <td className="px-4 py-2">John Doe</td>
                <td className="px-4 py-2">john@example.com</td>
                <td className="px-4 py-2">Admin</td>
              </tr>
              <tr>
                <td className="px-4 py-2">Jane Smith</td>
                <td className="px-4 py-2">jane@example.com</td>
                <td className="px-4 py-2">User</td>
              </tr>
              <tr>
                <td className="px-4 py-2">Bob Wilson</td>
                <td className="px-4 py-2">bob@example.com</td>
                <td className="px-4 py-2">User</td>
              </tr>
            </tbody>
          </table>
          <div className="flex items-center justify-between px-4 py-3 border-t border-nest-border bg-nest-surface">
            <p className="text-sm text-nest-muted">Showing 1-3 of 30 results</p>
            <Pagination count={10} page={page3} onChange={(_, p) => setPage3(p)} />
          </div>
        </div>
      </section>
    </div>
  );
}
