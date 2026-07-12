import { useState } from 'react';
import { Accordion, AccordionItem } from './Accordion';

export function AccordionDemos() {
  const [expanded, setExpanded] = useState('panel1');
  const [multiExpanded, setMultiExpanded] = useState<string[]>(['faq1']);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <Accordion defaultExpanded="panel1">
          <AccordionItem value="panel1" summary="Panel 1 Title">
            <p className="text-nest-muted">This is the content of panel 1. It can contain any content.</p>
          </AccordionItem>
          <AccordionItem value="panel2" summary="Panel 2 Title">
            <p className="text-nest-muted">This is the content of panel 2. Click the summaries to expand/collapse.</p>
          </AccordionItem>
          <AccordionItem value="panel3" summary="Panel 3 Title">
            <p className="text-nest-muted">This is the content of panel 3. Multiple panels can be expanded at once.</p>
          </AccordionItem>
        </Accordion>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Exclusive (Single Expanded)</h2>
        <Accordion expanded={expanded} onChange={(v) => setExpanded(v as string)} exclusive>
          <AccordionItem value="panel1" summary="Only One at a Time #1">
            <p className="text-nest-muted">When one panel expands, others collapse automatically.</p>
          </AccordionItem>
          <AccordionItem value="panel2" summary="Only One at a Time #2">
            <p className="text-nest-muted">This is useful for accordions where you want to focus on one section.</p>
          </AccordionItem>
          <AccordionItem value="panel3" summary="Only One at a Time #3">
            <p className="text-nest-muted">Set exclusive=true to enable this behavior.</p>
          </AccordionItem>
        </Accordion>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Multiple Expanded</h2>
        <Accordion expanded={multiExpanded} onChange={(v) => setMultiExpanded(v as string[])} exclusive={false}>
          <AccordionItem value="faq1" summary="FAQ Question 1">
            <p className="text-nest-muted">Answer to question 1. Multiple panels can be open simultaneously.</p>
          </AccordionItem>
          <AccordionItem value="faq2" summary="FAQ Question 2">
            <p className="text-nest-muted">Answer to question 2. This is great for FAQ sections.</p>
          </AccordionItem>
          <AccordionItem value="faq3" summary="FAQ Question 3">
            <p className="text-nest-muted">Answer to question 3. Users can keep relevant sections visible.</p>
          </AccordionItem>
        </Accordion>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <Accordion disabled>
          <AccordionItem value="panel1" summary="Disabled Panel 1">
            <p className="text-nest-muted">This panel cannot be expanded or collapsed.</p>
          </AccordionItem>
          <AccordionItem value="panel2" summary="Disabled Panel 2">
            <p className="text-nest-muted">All panels in this accordion are disabled.</p>
          </AccordionItem>
        </Accordion>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Individual Panel Disabled</h2>
        <Accordion defaultExpanded="">
          <AccordionItem value="panel1" summary="Enabled Panel">
            <p className="text-nest-muted">This panel can be toggled.</p>
          </AccordionItem>
          <AccordionItem value="panel2" summary="Disabled Panel" disabled>
            <p className="text-nest-muted">This specific panel is disabled while others work.</p>
          </AccordionItem>
          <AccordionItem value="panel3" summary="Another Enabled Panel">
            <p className="text-nest-muted">This panel can also be toggled.</p>
          </AccordionItem>
        </Accordion>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Rich Content</h2>
        <Accordion defaultExpanded="rich1">
          <AccordionItem
            value="rich1"
            summary={
              <div className="flex items-center gap-3">
                <span className="flex h-8 w-8 items-center justify-center rounded-full bg-nest-primary text-white text-sm">1</span>
                <span>Step One: Planning</span>
              </div>
            }
          >
            <div className="pl-11">
              <p className="text-nest-muted">Detailed content about planning your project. Can include lists, images, or any HTML.</p>
              <ul className="mt-2 list-disc pl-5 text-nest-muted">
                <li>Define requirements</li>
                <li>Create timeline</li>
                <li>Allocate resources</li>
              </ul>
            </div>
          </AccordionItem>
          <AccordionItem
            value="rich2"
            summary={
              <div className="flex items-center gap-3">
                <span className="flex h-8 w-8 items-center justify-center rounded-full bg-nest-secondary text-white text-sm">2</span>
                <span>Step Two: Execution</span>
              </div>
            }
          >
            <div className="pl-11">
              <p className="text-nest-muted">Detailed content about executing your plan. The summary can include any custom JSX.</p>
            </div>
          </AccordionItem>
        </Accordion>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Settings Sections</h2>
        <Accordion defaultExpanded="account">
          <AccordionItem value="account" summary="Account Settings">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm">Email Notifications</span>
                <input type="checkbox" defaultChecked className="h-4 w-4" />
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Two-Factor Authentication</span>
                <input type="checkbox" className="h-4 w-4" />
              </div>
            </div>
          </AccordionItem>
          <AccordionItem value="privacy" summary="Privacy Settings">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm">Profile Visibility</span>
                <select className="text-sm border rounded px-2 py-1">
                  <option>Public</option>
                  <option>Friends Only</option>
                  <option>Private</option>
                </select>
              </div>
            </div>
          </AccordionItem>
        </Accordion>
      </section>
    </div>
  );
}
