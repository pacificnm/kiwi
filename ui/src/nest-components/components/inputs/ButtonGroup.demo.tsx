import { ButtonGroup } from './ButtonGroup';
import { Button } from './Button';

export function ButtonGroupDemos() {
  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variants</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Text (default)</p>
            <ButtonGroup>
              <Button>One</Button>
              <Button>Two</Button>
              <Button>Three</Button>
            </ButtonGroup>
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Outlined</p>
            <ButtonGroup variant="outlined">
              <Button>One</Button>
              <Button>Two</Button>
              <Button>Three</Button>
            </ButtonGroup>
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Contained</p>
            <ButtonGroup variant="contained">
              <Button>One</Button>
              <Button>Two</Button>
              <Button>Three</Button>
            </ButtonGroup>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Primary (default)</p>
            <ButtonGroup variant="contained">
              <Button>One</Button>
              <Button>Two</Button>
            </ButtonGroup>
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Secondary</p>
            <ButtonGroup variant="contained" color="secondary">
              <Button>One</Button>
              <Button>Two</Button>
            </ButtonGroup>
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Success</p>
            <ButtonGroup variant="contained" color="success">
              <Button>One</Button>
              <Button>Two</Button>
            </ButtonGroup>
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Error</p>
            <ButtonGroup variant="contained" color="error">
              <Button>One</Button>
              <Button>Two</Button>
            </ButtonGroup>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Small</p>
            <ButtonGroup size="small" variant="outlined">
              <Button>One</Button>
              <Button>Two</Button>
              <Button>Three</Button>
            </ButtonGroup>
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Medium (default)</p>
            <ButtonGroup size="medium" variant="outlined">
              <Button>One</Button>
              <Button>Two</Button>
              <Button>Three</Button>
            </ButtonGroup>
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Large</p>
            <ButtonGroup size="large" variant="outlined">
              <Button>One</Button>
              <Button>Two</Button>
              <Button>Three</Button>
            </ButtonGroup>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Orientation</h2>
        <div className="flex gap-8">
          <div>
            <p className="text-sm text-nest-muted mb-2">Horizontal (default)</p>
            <ButtonGroup variant="contained">
              <Button>Top</Button>
              <Button>Middle</Button>
              <Button>Bottom</Button>
            </ButtonGroup>
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Vertical</p>
            <ButtonGroup orientation="vertical" variant="contained">
              <Button>Top</Button>
              <Button>Middle</Button>
              <Button>Bottom</Button>
            </ButtonGroup>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Full Width</h2>
        <div className="w-64">
          <ButtonGroup fullWidth variant="outlined">
            <Button>Stretch</Button>
            <Button>Across</Button>
            <Button>Container</Button>
          </ButtonGroup>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Icons</h2>
        <div>
          <p className="text-sm text-nest-muted mb-2">Icon buttons</p>
          <ButtonGroup variant="outlined">
            <Button aria-label="cut">✂️</Button>
            <Button aria-label="copy">📋</Button>
            <Button aria-label="paste">📄</Button>
          </ButtonGroup>
        </div>
      </section>
    </div>
  );
}
