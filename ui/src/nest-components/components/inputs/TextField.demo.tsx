import React, { useState } from 'react';
import { TextField } from './TextField';
import { Search, Mail, Lock, Eye, EyeOff } from 'lucide-react';

/**
 * TextField Component Demos
 */

export function TextFieldDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Basic Usage */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <div className="space-y-4">
          <TextField label="First Name" placeholder="Enter your first name" />
          <TextField label="Last Name" placeholder="Enter your last name" />
        </div>
      </section>

      {/* Variants Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variants</h2>
        <div className="space-y-4">
          <TextField label="Outlined" variant="outlined" placeholder="Outlined variant" />
          <TextField label="Filled" variant="filled" placeholder="Filled variant" />
          <TextField label="Standard" variant="standard" placeholder="Standard variant" />
        </div>
      </section>

      {/* Sizes Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <TextField label="Small" size="small" placeholder="Small input" />
          <TextField label="Medium" size="medium" placeholder="Medium input" />
        </div>
      </section>

      {/* With Icons (Adornments) */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Adornments</h2>
        <div className="space-y-4">
          <TextField
            label="Search"
            placeholder="Search..."
            startAdornment={<Search className="size-4" />}
          />
          <TextField
            label="Email"
            type="email"
            placeholder="you@example.com"
            startAdornment={<Mail className="size-4" />}
          />
          <PasswordInput />
        </div>
      </section>

      {/* Error States */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Validation & Errors</h2>
        <div className="space-y-4">
          <TextField
            label="Username"
            defaultValue="taken_username"
            error="This username is already taken"
          />
          <TextField
            label="Email"
            placeholder="Enter email"
            helperText="We'll never share your email"
          />
        </div>
      </section>

      {/* Multiline */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Multiline</h2>
        <TextField
          label="Description"
          placeholder="Enter a detailed description..."
          multiline
          rows={4}
          fullWidth
        />
      </section>

      {/* Full Width */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Full Width</h2>
        <div className="space-y-4">
          <TextField label="Full Width Input" fullWidth placeholder="Spans container width" />
          <TextField
            label="With Helper"
            fullWidth
            helperText="This input spans the full container width"
          />
        </div>
      </section>

      {/* Interactive Form Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Interactive Form</h2>
        <InteractiveForm />
      </section>
    </div>
  );
}

function PasswordInput() {
  const [showPassword, setShowPassword] = useState(false);

  return (
    <TextField
      label="Password"
      type={showPassword ? 'text' : 'password'}
      placeholder="Enter password"
      startAdornment={<Lock className="size-4" />}
      endAdornment={
        <button
          type="button"
          onClick={() => setShowPassword(!showPassword)}
          className="text-nest-muted hover:text-nest-foreground"
          tabIndex={-1}
        >
          {showPassword ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
        </button>
      }
    />
  );
}

function InteractiveForm() {
  const [formData, setFormData] = useState({
    name: '',
    email: '',
    message: '',
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = () => {
    const newErrors: Record<string, string> = {};
    if (!formData.name.trim()) newErrors.name = 'Name is required';
    if (!formData.email.includes('@')) newErrors.email = 'Valid email is required';
    if (formData.message.length < 10) newErrors.message = 'Message must be at least 10 characters';
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validate()) {
      alert('Form submitted!');
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <TextField
        label="Name"
        value={formData.name}
        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
        error={errors.name}
        fullWidth
      />
      <TextField
        label="Email"
        type="email"
        value={formData.email}
        onChange={(e) => setFormData({ ...formData, email: e.target.value })}
        error={errors.email}
        fullWidth
      />
      <TextField
        label="Message"
        multiline
        rows={3}
        value={formData.message}
        onChange={(e) => setFormData({ ...formData, message: e.target.value })}
        error={errors.message}
        fullWidth
      />
      <Button type="submit" variant="contained" color="primary">
        Submit
      </Button>
    </form>
  );
}

// Import Button for the interactive form
import { Button } from './Button';
