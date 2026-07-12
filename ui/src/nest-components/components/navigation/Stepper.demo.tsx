import { useState } from 'react';
import { Stepper, Step, StepLabel } from './Stepper';
import { Button } from '../inputs/Button';

export function StepperDemos() {
  const [activeStep, setActiveStep] = useState(0);
  const [verticalStep, setVerticalStep] = useState(0);

  const steps = ['Select Campaign', 'Configure Settings', 'Review', 'Complete'];
  const verticalSteps = ['Account Setup', 'Profile Details', 'Verification', 'Preferences'];

  const handleNext = () => setActiveStep((prev) => Math.min(prev + 1, steps.length - 1));
  const handleBack = () => setActiveStep((prev) => Math.max(prev - 1, 0));
  const handleReset = () => setActiveStep(0);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Horizontal Stepper</h2>
        <Stepper activeStep={activeStep}>
          {steps.map((label) => (
            <Step key={label}>
              <StepLabel>{label}</StepLabel>
            </Step>
          ))}
        </Stepper>
        <div className="mt-4 flex gap-2">
          <Button onClick={handleBack} disabled={activeStep === 0}>Back</Button>
          <Button onClick={handleNext} disabled={activeStep === steps.length - 1}>Next</Button>
          <Button onClick={handleReset} variant="outlined">Reset</Button>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Vertical Stepper</h2>
        <div className="max-w-md">
          <Stepper activeStep={verticalStep} orientation="vertical">
            {verticalSteps.map((label) => (
              <Step key={label}>
                <StepLabel>{label}</StepLabel>
              </Step>
            ))}
          </Stepper>
          <div className="mt-4 flex gap-2">
            <Button onClick={() => setVerticalStep((p) => Math.max(p - 1, 0))} disabled={verticalStep === 0}>Back</Button>
            <Button onClick={() => setVerticalStep((p) => Math.min(p + 1, verticalSteps.length - 1))} disabled={verticalStep === verticalSteps.length - 1}>Next</Button>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Completed Steps</h2>
        <Stepper activeStep={2}>
          <Step completed>
            <StepLabel completed>Account Created</StepLabel>
          </Step>
          <Step completed>
            <StepLabel completed>Email Verified</StepLabel>
          </Step>
          <Step>
            <StepLabel active>Profile Setup</StepLabel>
          </Step>
          <Step>
            <StepLabel>Preferences</StepLabel>
          </Step>
        </Stepper>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Linear Progress</h2>
        <Stepper activeStep={1}>
          <Step completed>
            <StepLabel completed>Step 1: Research</StepLabel>
          </Step>
          <Step>
            <StepLabel active>Step 2: Design</StepLabel>
          </Step>
          <Step>
            <StepLabel>Step 3: Develop</StepLabel>
          </Step>
          <Step>
            <StepLabel>Step 4: Deploy</StepLabel>
          </Step>
        </Stepper>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Onboarding Flow</h2>
        <div className="border border-nest-border rounded-nest-md p-6 bg-nest-surface">
          <Stepper activeStep={activeStep}>
            {steps.map((label) => (
              <Step key={label}>
                <StepLabel>{label}</StepLabel>
              </Step>
            ))}
          </Stepper>
          <div className="mt-6 p-4 border border-nest-border rounded-nest-md bg-nest-background">
            <h3 className="text-lg font-semibold mb-2">{steps[activeStep]}</h3>
            <p className="text-nest-muted mb-4">
              {activeStep === 0 && "Select a campaign from the list or create a new one."}
              {activeStep === 1 && "Configure your campaign settings and targeting options."}
              {activeStep === 2 && "Review your configuration before proceeding."}
              {activeStep === 3 && "Your campaign is ready to launch!"}
            </p>
            <div className="flex gap-2">
              <Button onClick={handleBack} disabled={activeStep === 0}>Back</Button>
              <Button onClick={handleNext} disabled={activeStep === steps.length - 1}>
                {activeStep === steps.length - 2 ? 'Finish' : 'Next'}
              </Button>
            </div>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">All Completed</h2>
        <Stepper activeStep={3}>
          <Step completed>
            <StepLabel completed>First Step</StepLabel>
          </Step>
          <Step completed>
            <StepLabel completed>Second Step</StepLabel>
          </Step>
          <Step completed>
            <StepLabel completed>Third Step</StepLabel>
          </Step>
        </Stepper>
      </section>
    </div>
  );
}
