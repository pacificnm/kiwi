import { Card, CardHeader, CardContent, CardActions, CardMedia } from './Card';
import { Button } from '../inputs/Button';
import { IconButton } from '../inputs/IconButton';

/**
 * Card Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function CardDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Basic Card */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Card</h2>
        <Card className="max-w-sm">
          <CardContent>
            <p className="text-nest-foreground">
              This is a basic card with just content. Cards are great for grouping
              related information together.
            </p>
          </CardContent>
        </Card>
      </section>

      {/* Card with Header and Actions */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Card with Header and Actions</h2>
        <Card className="max-w-sm">
          <CardHeader
            title="Card Title"
            subheader="Card subtitle or description"
          />
          <CardContent>
            <p className="text-nest-muted">
              Card content goes here. This could be a description, article text,
              or any other content that needs to be displayed.
            </p>
          </CardContent>
          <CardActions>
            <Button size="small">Learn More</Button>
            <Button size="small" variant="outlined">Cancel</Button>
          </CardActions>
        </Card>
      </section>

      {/* Card with Avatar */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Card with Avatar</h2>
        <Card className="max-w-sm">
          <CardHeader
            avatar={
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-nest-primary text-white font-bold">
                JD
              </div>
            }
            title="John Doe"
            subheader="Software Engineer"
            action={
              <IconButton size="small" aria-label="settings">
                ⋮
              </IconButton>
            }
          />
          <CardContent>
            <p className="text-nest-muted">
              User bio or description. Cards with avatars are great for user
              profiles, comments, or contact information.
            </p>
          </CardContent>
        </Card>
      </section>

      {/* Card with Media */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Card with Media</h2>
        <div className="grid gap-4 md:grid-cols-2">
          <Card className="max-w-sm">
            <CardMedia
              image="https://picsum.photos/400/200"
              alt="Landscape"
              height="160px"
            />
            <CardHeader title="Image Card" />
            <CardContent>
              <p className="text-nest-muted">
                Cards with images are perfect for articles, products, or media
                galleries.
              </p>
            </CardContent>
            <CardActions>
              <Button size="small">View</Button>
            </CardActions>
          </Card>

          <Card className="max-w-sm">
            <CardMedia
              className="flex items-center justify-center bg-gradient-to-br from-nest-primary/20 to-nest-secondary/20"
              height="160px"
            >
              <span className="text-4xl">🎨</span>
            </CardMedia>
            <CardHeader title="Placeholder Media" />
            <CardContent>
              <p className="text-nest-muted">
                Custom media divs work great for icons, gradients, or embedded
                content.
              </p>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* Horizontal Card */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Horizontal Card</h2>
        <Card className="max-w-md">
          <div className="flex">
            <CardMedia
              image="https://picsum.photos/150/200"
              alt="Thumbnail"
              className="!h-auto !w-40"
            />
            <div className="flex flex-1 flex-col">
              <CardHeader
                title="Horizontal Layout"
                subheader="Side-by-side content"
                className="flex-1"
              />
              <CardActions>
                <Button size="small">Action</Button>
              </CardActions>
            </div>
          </div>
        </Card>
      </section>

      {/* Elevation Variations */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Elevation Variations</h2>
        <div className="grid gap-4 md:grid-cols-5">
          <Card elevation={0} className="p-4 text-center">
            <p className="font-medium">elevation={0}</p>
          </Card>
          <Card elevation={1} className="p-4 text-center">
            <p className="font-medium">elevation={1}</p>
          </Card>
          <Card elevation={2} className="p-4 text-center">
            <p className="font-medium">elevation={2}</p>
          </Card>
          <Card elevation={3} className="p-4 text-center">
            <p className="font-medium">elevation={3}</p>
          </Card>
          <Card elevation={4} className="p-4 text-center">
            <p className="font-medium">elevation={4}</p>
          </Card>
        </div>
      </section>

      {/* Outlined Card */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Outlined Variant</h2>
        <Card variant="outlined" className="max-w-sm">
          <CardHeader title="Outlined Card" />
          <CardContent>
            <p className="text-nest-muted">
              Cards with variant="outlined" use a border instead of shadow.
              Great for subtle content containers.
            </p>
          </CardContent>
        </Card>
      </section>

      {/* Interactive Card */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Interactive Card</h2>
        <Card
          component="button"
          className="max-w-sm cursor-pointer text-left transition-all hover:shadow-md hover:shadow-nest-primary/20"
          onClick={() => {
            const event = new CustomEvent('card-click');
            window.dispatchEvent(event);
          }}
        >
          <CardHeader title="Clickable Card" subheader="Click me" />
          <CardContent>
            <p className="text-nest-muted">
              This entire card is clickable. Perfect for link cards or
              selectable items.
            </p>
          </CardContent>
        </Card>
      </section>

      {/* Card Grid Layout */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Card Grid Layout</h2>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {[1, 2, 3, 4, 5, 6].map((i) => (
            <Card key={i} elevation={2}>
              <CardMedia
                image={`https://picsum.photos/300/150?random=${i}`}
                alt={`Card ${i}`}
                height="120px"
              />
              <CardHeader
                title={`Card ${i}`}
                subheader={`Description ${i}`}
              />
              <CardContent>
                <p className="text-sm text-nest-muted">
                  Card content for item {i}. Cards work great in responsive
                  grids.
                </p>
              </CardContent>
              <CardActions>
                <Button size="small">View</Button>
              </CardActions>
            </Card>
          ))}
        </div>
      </section>

      {/* Complex Card */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Complex Card</h2>
        <Card className="max-w-md">
          <CardMedia
            image="https://picsum.photos/600/250"
            alt="Featured"
            height="200px"
          />
          <CardHeader
            title="Featured Article"
            subheader="Published on July 11, 2026"
            avatar={
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-nest-accent text-white font-bold">
                FA
              </div>
            }
            action={
              <IconButton size="small" aria-label="bookmark">
                ☆
              </IconButton>
            }
          />
          <CardContent>
            <p className="text-nest-muted">
              This is a more complex card example showcasing all Card components
              working together. The combination of media, header with avatar and
              action, content, and action buttons creates a rich, interactive
              component.
            </p>
          </CardContent>
          <CardActions disableSpacing>
            <Button size="small" variant="contained">Read</Button>
            <Button size="small" variant="outlined">Share</Button>
            <Button size="small" variant="text">Save</Button>
          </CardActions>
        </Card>
      </section>
    </div>
  );
}
