import { FormControl, FormLabel, FormHelperText } from './FormControl';
import { TextField } from './TextField';

export function FormControlDemos() {
  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <div className="w-64 space-y-4">
          <FormControl>
            <FormLabel htmlFor="name">Name</FormLabel>
            <TextField id="name" placeholder="Enter your name" />
            <FormHelperText>Enter your full name</FormHelperText>
          </FormControl>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Required Field</h2>
        <div className="w-64 space-y-4">
          <FormControl required>
            <FormLabel htmlFor="email" required>Email</FormLabel>
            <TextField id="email" type="email" placeholder="Enter email" required />
            <FormHelperText>We'll never share your email</FormHelperText>
          </FormControl>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Error State</h2>
        <div className="w-64 space-y-4">
          <FormControl error>
            <FormLabel error htmlFor="email-error">Email</FormLabel>
            <TextField id="email-error" type="email" placeholder="Enter email" defaultValue="invalid" />
            <FormHelperText error>Please enter a valid email address</FormHelperText>
          </FormControl>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <div className="w-64 space-y-4">
          <FormControl disabled>
            <FormLabel disabled htmlFor="disabled-input">Disabled Field</FormLabel>
            <TextField id="disabled-input" placeholder="Disabled" disabled />
            <FormHelperText disabled>This field is disabled</FormHelperText>
          </FormControl>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Focused State</h2>
        <div className="w-64 space-y-4">
          <FormControl focused>
            <FormLabel focused htmlFor="focused-input">Focused Label</FormLabel>
            <TextField id="focused-input" placeholder="Focused" />
            <FormHelperText>This field is focused</FormHelperText>
          </FormControl>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Full Width</h2>
        <div className="w-64 space-y-4">
          <FormControl fullWidth>
            <FormLabel htmlFor="full-width">Full Width Field</FormLabel>
            <TextField id="full-width" placeholder="Takes full width" />
            <FormHelperText>This field stretches</FormHelperText>
          </FormControl>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Helper Text Variants</h2>
        <div className="w-64 space-y-4">
          <FormControl>
            <FormLabel htmlFor="password">Password</FormLabel>
            <TextField id="password" type="password" placeholder="Enter password" />
            <FormHelperText>Must be at least 8 characters</FormHelperText>
          </FormControl>

          <FormControl>
            <FormLabel htmlFor="confirm-password">Confirm Password</FormLabel>
            <TextField id="confirm-password" type="password" placeholder="Confirm password" />
            <FormHelperText visuallyHidden>This field is required for accessibility</FormHelperText>
          </FormControl>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Complete Form Example</h2>
        <div className="w-64 space-y-4 border border-nest-border rounded-nest-md p-4">
          <FormControl>
            <FormLabel htmlFor="form-name">Name</FormLabel>
            <TextField id="form-name" placeholder="John Doe" />
          </FormControl>

          <FormControl>
            <FormLabel htmlFor="form-email">Email</FormLabel>
            <TextField id="form-email" type="email" placeholder="john@example.com" />
            <FormHelperText>We'll send a confirmation email</FormHelperText>
          </FormControl>

          <FormControl>
            <FormLabel htmlFor="form-message">Message</FormLabel>
            <TextField id="form-message" multiline rows={3} placeholder="Your message" />
          </FormControl>
        </div>
      </section>
    </div>
  );
}
