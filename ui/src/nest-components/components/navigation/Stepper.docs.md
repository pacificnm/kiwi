# Stepper, Step, StepLabel, StepIcon

A component for displaying progress through a sequence of steps.

## When to Use

Use Stepper when:
- Users need to complete a multi-step process (wizard, onboarding)
- Showing progress through a timeline or workflow
- Breaking complex tasks into manageable chunks
- Users need to understand where they are in a process

## Props

### Stepper

| Prop          | Type                        | Default       | Description                           |
|---------------|-----------------------------|---------------|---------------------------------------|
| `activeStep`  | `number`                    | `0`           | The currently active step (0-indexed) |
| `orientation` | `'horizontal' \| 'vertical'`| `'horizontal'`| The orientation of the stepper        |
| `className`   | `string`                    | -             | Additional CSS classes                |

### Step

| Prop       | Type      | Default | Description                    |
|------------|-----------|---------|--------------------------------|
| `completed`| `boolean` | -       | If true, the step is completed |
| `disabled` | `boolean` | `false` | If true, the step is disabled  |
| `className`| `string`  | -       | Additional CSS classes         |

### StepLabel

| Prop       | Type      | Default | Description                    |
|------------|-----------|---------|--------------------------------|
| `completed`| `boolean` | -       | If true, shows completed state |
| `active`   | `boolean` | -       | If true, shows active state    |
| `disabled` | `boolean` | `false` | If true, the label is disabled |
| `className`| `string`  | -       | Additional CSS classes         |

### StepIcon

| Prop       | Type      | Default | Description                    |
|------------|-----------|---------|--------------------------------|
| `completed`| `boolean` | -       | If true, shows check icon      |
| `active`   | `boolean` | -       | If true, shows active styling  |
| `disabled` | `boolean` | `false` | If true, the icon is disabled  |

## Examples

### Basic Usage

```tsx
import { Stepper, Step, StepLabel } from '@nest/components';

<Stepper activeStep={1}>
  <Step>
    <StepLabel>Step 1</StepLabel>
  </Step>
  <Step>
    <StepLabel>Step 2</StepLabel>
  </Step>
  <Step>
    <StepLabel>Step 3</StepLabel>
  </Step>
</Stepper>
```

### With Completed Steps

```tsx
import { Stepper, Step, StepLabel } from '@nest/components';

<Stepper activeStep={2}>
  <Step completed>
    <StepLabel completed>Account Created</StepLabel>
  </Step>
  <Step completed>
    <StepLabel completed>Email Verified</StepLabel>
  </Step>
  <Step active>
    <StepLabel active>Profile Setup</StepLabel>
  </Step>
  <Step>
    <StepLabel>Preferences</StepLabel>
  </Step>
</Stepper>
```

### Vertical Orientation

```tsx
import { Stepper, Step, StepLabel } from '@nest/components';

<Stepper activeStep={1} orientation="vertical">
  <Step>
    <StepLabel>Step 1</StepLabel>
  </Step>
  <Step>
    <StepLabel>Step 2</StepLabel>
  </Step>
  <Step>
    <StepLabel>Step 3</StepLabel>
  </Step>
</Stepper>
```

### Interactive Wizard

```tsx
import { useState } from 'react';
import { Stepper, Step, StepLabel } from '@nest/components';
import { Button } from '@nest/components';

function Wizard() {
  const [activeStep, setActiveStep] = useState(0);
  const steps = ['Select Plan', 'Enter Details', 'Payment', 'Complete'];

  const handleNext = () => setActiveStep((prev) => prev + 1);
  const handleBack = () => setActiveStep((prev) => prev - 1);

  return (
    <div>
      <Stepper activeStep={activeStep}>
        {steps.map((label) => (
          <Step key={label}>
            <StepLabel>{label}</StepLabel>
          </Step>
        ))}
      </Stepper>
      <div className="mt-4 flex gap-2">
        <Button onClick={handleBack} disabled={activeStep === 0}>
          Back
        </Button>
        <Button onClick={handleNext} disabled={activeStep === steps.length - 1}>
          {activeStep === steps.length - 2 ? 'Finish' : 'Next'}
        </Button>
      </div>
    </div>
  );
}
```

### Onboarding Flow

```tsx
import { Stepper, Step, StepLabel } from '@nest/components';

function OnboardingFlow() {
  return (
    <div className="border rounded-lg p-6">
      <Stepper activeStep={currentStep}>
        <Step completed>
          <StepLabel completed>Welcome</StepLabel>
        </Step>
        <Step active>
          <StepLabel active>Account Setup</StepLabel>
        </Step>
        <Step>
          <StepLabel>Profile</StepLabel>
        </Step>
        <Step>
          <StepLabel>Complete</StepLabel>
        </Step>
      </Stepper>
      <div className="mt-6">
        {/* Step content here */}
      </div>
    </div>
  );
}
```

## Accessibility

- Stepper uses `role="progressbar"` with `aria-valuenow` for the current step
- Each step should have a meaningful label
- Active step should be clearly indicated visually
- Consider providing text alternatives for the step indicator

## Tips

- Use `activeStep` to control which step is currently active
- Use `completed` prop on Step/StepLabel to show checkmark for completed steps
- For linear progress, disable navigation to incomplete steps
- For non-linear progress, allow users to jump between steps
- Use vertical orientation for sidebars or narrow layouts
