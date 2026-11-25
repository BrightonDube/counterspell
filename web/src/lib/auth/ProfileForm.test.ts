import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor
} from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ProfileForm from './ProfileForm.svelte';
import { patchProfile } from './client.js';
import { session } from './session.svelte.js';

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

// CPX-3: the shared profile form labels display_name as "Full name",
// offers a searchable country picker that stores ISO alpha-2, supports
// clearing and explicit cancel, and keeps legacy codes untouched.
const baseUser = {
  display_name: 'Witch Hunter',
  country: 'ZA',
  avatar_url: '/art/creatures/skull-green-eyes.webp'
};

function mustUser() {
  if (!session.user) throw new Error('session.user unset');
  return session.user;
}

describe('ProfileForm', () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    // Saves mutate the shared session mock — restore the fixture.
    session.user = { ...baseUser };
    session.status = 'signed_in';
  });

  it('labels the name field "Full name" and offers a country picker', async () => {
    render(ProfileForm);

    expect(screen.getByLabelText('Full name')).toBeTruthy();
    const country = screen.getByLabelText('Country');
    expect(country.getAttribute('list')).toBe('profile-countries');
    // The picker lists readable country names, not raw codes.
    const options = document.querySelectorAll('#profile-countries option');
    const names = Array.from(options).map((o) => o.getAttribute('value'));
    expect(names).toContain('South Africa');
    expect(names).toContain('United Kingdom');
    // Current value (ZA) is prefilled as a readable name.
    expect((country as HTMLInputElement).value).toBe('South Africa');
  });

  it('saves a searched country name as its alpha-2 code', async () => {
    render(ProfileForm);

    await fireEvent.input(screen.getByLabelText('Country'), {
      target: { value: 'Germany' }
    });
    await fireEvent.input(screen.getByLabelText('Full name'), {
      target: { value: 'Night Walker' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));

    await waitFor(() => expect(patchProfile).toHaveBeenCalledOnce());
    expect(patchProfile).toHaveBeenCalledWith({
      display_name: 'Night Walker',
      country: 'DE'
    });
  });

  it('accepts a bare two-letter code and keeps legacy codes verbatim', async () => {
    render(ProfileForm);

    await fireEvent.input(screen.getByLabelText('Country'), {
      target: { value: 'za' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));
    await waitFor(() =>
      expect(patchProfile).toHaveBeenCalledWith({
        display_name: 'Witch Hunter',
        country: 'ZA'
      })
    );

    // An unrecognised legacy code prefilled from the session round-trips
    // untouched instead of being silently rewritten (shown uppercased).
    mustUser().country = 'zz';
    const { unmount } = render(ProfileForm);
    const country = screen
      .getAllByLabelText('Country')
      .at(-1) as HTMLInputElement;
    expect(country.value).toBe('ZZ');
    const saveButtons = screen.getAllByRole('button', {
      name: 'Save profile'
    });
    const saveButton = saveButtons[saveButtons.length - 1];
    await fireEvent.click(saveButton);
    // The earlier save in this test persisted 'Night Walker' — the
    // second form mounts with that as its current name.
    await waitFor(() =>
      expect(patchProfile).toHaveBeenLastCalledWith({
        display_name: 'Night Walker',
        country: 'zz'
      })
    );
    unmount();
    mustUser().country = 'ZA';
  });

  it('sends null when the country is cleared', async () => {
    render(ProfileForm);

    await fireEvent.input(screen.getByLabelText('Country'), {
      target: { value: '' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));
    await waitFor(() =>
      expect(patchProfile).toHaveBeenCalledWith({
        display_name: 'Witch Hunter',
        country: null
      })
    );
  });

  it('rejects an unrecognised country without calling the API', async () => {
    render(ProfileForm);

    await fireEvent.input(screen.getByLabelText('Country'), {
      target: { value: 'Atlantis' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));
    expect(await screen.findByRole('alert')).toBeTruthy();
    expect(patchProfile).not.toHaveBeenCalled();
  });

  it('trims the full name and blocks empty saves', async () => {
    render(ProfileForm);

    await fireEvent.input(screen.getByLabelText('Full name'), {
      target: { value: '  Night Walker  ' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));
    await waitFor(() =>
      expect(patchProfile).toHaveBeenCalledWith({
        display_name: 'Night Walker',
        country: 'ZA'
      })
    );

    await fireEvent.input(screen.getByLabelText('Full name'), {
      target: { value: '   ' }
    });
    expect(screen.getByRole('button', { name: 'Save profile' })).toHaveProperty(
      'disabled',
      true
    );
  });

  it('cancel restores the saved values and clears feedback', async () => {
    render(ProfileForm);

    const name = screen.getByLabelText('Full name') as HTMLInputElement;
    const country = screen.getByLabelText('Country') as HTMLInputElement;
    await fireEvent.input(name, { target: { value: 'Someone Else' } });
    await fireEvent.input(country, { target: { value: 'Germany' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(name.value).toBe('Witch Hunter');
    expect(country.value).toBe('South Africa');
    expect(screen.queryByRole('status')).toBeNull();
    expect(screen.queryByRole('alert')).toBeNull();
    expect(patchProfile).not.toHaveBeenCalled();
  });

  it('prevents duplicate save requests while one is in flight', async () => {
    let resolvePatch: ((v: unknown) => void) | undefined;
    vi.mocked(patchProfile).mockImplementationOnce(
      () => new Promise((r) => (resolvePatch = r))
    );
    render(ProfileForm);

    const save = screen.getByRole('button', { name: 'Save profile' });
    await fireEvent.click(save);
    await fireEvent.click(save);
    expect(patchProfile).toHaveBeenCalledTimes(1);
    resolvePatch?.({
      user: { display_name: 'Witch Hunter', country: 'ZA' },
      capabilities: { edit_profile: true }
    });
    await waitFor(() => expect(screen.findByRole('status')).toBeTruthy());
  });

  it('keeps the draft and shows an error when saving fails', async () => {
    vi.mocked(patchProfile).mockRejectedValueOnce(new Error('nope'));
    render(ProfileForm);

    const name = screen.getByLabelText('Full name') as HTMLInputElement;
    await fireEvent.input(name, { target: { value: 'Someone Else' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));

    expect(await screen.findByRole('alert')).toBeTruthy();
    expect(name.value).toBe('Someone Else');
    expect(mustUser().display_name).toBe('Witch Hunter');
  });
});
