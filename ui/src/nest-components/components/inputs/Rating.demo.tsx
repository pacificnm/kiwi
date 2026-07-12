import { useState } from 'react';
import { Rating } from './Rating';

export function RatingDemos() {
  const [value1, setValue1] = useState(3);
  const [value2, setValue2] = useState(4);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Uncontrolled</p>
            <Rating defaultValue={3} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Controlled</p>
            <Rating value={value1} onChange={(_, v) => setValue1(v)} />
            <p className="text-xs text-nest-muted mt-1">Value: {value1}</p>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Read-only</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Display only</p>
            <Rating value={4.5} readOnly />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Product rating</p>
            <div className="flex items-center gap-2">
              <Rating value={5} readOnly />
              <span className="text-sm text-nest-foreground">5.0 (128 reviews)</span>
            </div>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-4">
            <Rating defaultValue={3} size="small" />
            <span className="text-sm text-nest-muted">Small</span>
          </div>
          <div className="flex items-center gap-4">
            <Rating defaultValue={3} size="medium" />
            <span className="text-sm text-nest-muted">Medium</span>
          </div>
          <div className="flex items-center gap-4">
            <Rating defaultValue={3} size="large" />
            <span className="text-sm text-nest-muted">Large</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-4">
            <Rating defaultValue={4} color="primary" />
            <span className="text-sm">Primary</span>
          </div>
          <div className="flex items-center gap-4">
            <Rating defaultValue={4} color="secondary" />
            <span className="text-sm">Secondary</span>
          </div>
          <div className="flex items-center gap-4">
            <Rating defaultValue={4} color="warning" />
            <span className="text-sm">Warning (default)</span>
          </div>
          <div className="flex items-center gap-4">
            <Rating defaultValue={4} color="error" />
            <span className="text-sm">Error</span>
          </div>
          <div className="flex items-center gap-4">
            <Rating defaultValue={4} color="accent" />
            <span className="text-sm">Accent</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <Rating defaultValue={3} disabled />
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Max</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">10 stars</p>
            <Rating defaultValue={7} max={10} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">3 stars</p>
            <Rating defaultValue={2} max={3} size="large" />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Half Star Precision</h2>
        <div className="space-y-4">
          <Rating value={3.5} precision={0.5} onChange={(_, v) => setValue2(v)} />
          <p className="text-sm text-nest-muted">Value: {value2} (supports half stars)</p>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Review Form</h2>
        <div className="border border-nest-border rounded-nest-md p-4 max-w-sm space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Your Rating</label>
            <Rating defaultValue={0} />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Review</label>
            <textarea
              className="w-full border border-nest-border rounded-nest-md px-3 py-2 text-sm"
              rows={3}
              placeholder="Share your experience..."
            />
          </div>
          <button className="px-4 py-2 bg-nest-primary text-white rounded-nest-md text-sm font-medium">
            Submit Review
          </button>
        </div>
      </section>
    </div>
  );
}
