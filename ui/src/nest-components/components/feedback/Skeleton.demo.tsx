import { Skeleton } from './Skeleton';

export function SkeletonDemos() {
  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variants</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Text (default)</p>
            <Skeleton />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Circular</p>
            <Skeleton variant="circular" width={40} height={40} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Rectangular</p>
            <Skeleton variant="rectangular" width={200} height={100} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Rounded</p>
            <Skeleton variant="rounded" width={300} height={150} />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Small</p>
            <Skeleton width={100} height={20} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Medium</p>
            <Skeleton width={200} height={40} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">Large</p>
            <Skeleton width={400} height={80} />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Animation</h2>
        <div className="space-y-4">
          <div>
            <p className="text-sm text-nest-muted mb-2">Pulse (default)</p>
            <Skeleton width={200} height={30} />
          </div>
          <div>
            <p className="text-sm text-nest-muted mb-2">No animation</p>
            <Skeleton width={200} height={30} animation={false} />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Card Layout Example</h2>
        <div className="border border-nest-border rounded-nest-md p-4 w-64 space-y-3">
          <div className="flex items-center gap-3">
            <Skeleton variant="circular" width={40} height={40} />
            <div className="flex-1 space-y-2">
              <Skeleton width="60%" height={16} />
              <Skeleton width="40%" height={12} />
            </div>
          </div>
          <Skeleton variant="rounded" width="100%" height={100} />
          <div className="space-y-2">
            <Skeleton width="80%" height={12} />
            <Skeleton width="70%" height={12} />
          </div>
        </div>
      </section>
    </div>
  );
}
