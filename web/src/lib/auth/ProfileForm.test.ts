import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import ProfileForm from './ProfileForm.svelte';
import { patchProfile } from './client.js';

vi.mock('./client.js', () => ({
  patchProfile: vi.fn().mockResolvedValue({
    user: {
      display_name: 'Night Walker',
      country: 'ZA',
      avatar_url: '/art/creatures/skull-green-eyes.webp'
    },
    capabilities: { edit_profile: true }
  })
}));
vi.mock('./security.ts', () => ({ authProblem: () => 'Save failed.' }));
vi.mock('./session.svelte.js', () => ({
  session: {
    status: 'signed_in',
    user: {
      display_name: 'Witch Hunter',
      country: 'ZA',
      avatar_url: '/art/creatures/skull-green-eyes.webp'
    },
    capabilities: { edit_profile: true }
  }
}));

describe('ProfileForm', () => {
  it('saves display name and country through the profile seam', async () => {
    const saved = vi.fn();
    render(ProfileForm, { props: { onSaved: saved } });

    await fireEvent.input(screen.getByLabelText('Display name'), {
      target: { value: 'Night Walker' }
    });
    await fireEvent.input(screen.getByLabelText('Country code'), {
      target: { value: 'za' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));

    await waitFor(() => expect(saved).toHaveBeenCalledOnce());
    expect(patchProfile).toHaveBeenCalledWith({
      display_name: 'Night Walker',
      country: 'za'
    });
    expect((await screen.findByRole('status')).textContent).toBe(
      'Profile saved.'
    );
  });
});
