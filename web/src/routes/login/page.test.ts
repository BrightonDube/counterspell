import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor
} from '@testing-library/svelte';
import { afterEach, expect, it, vi } from 'vitest';
import Page from './+page.svelte';

vi.mock('$app/env', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/login') }
}));
vi.mock('#lib/auth/client.js', () => ({
  AuthApiError: class extends Error {},
  getConfig: vi.fn().mockResolvedValue({
    providers: { github: true, google: true },
    demo: { enabled: true, email: 'demo@example.co' }
  }),
  providerSignInUrl: (provider: string) => `/api/auth/${provider}/start`
}));
vi.mock('#lib/auth/security.ts', () => ({
  authProblem: () => 'Sign-in failed.',
  signInWithPasskey: vi.fn(),
  signInWithPassword: vi.fn(),
  supportsPasskeys: () => true,
  verifyTwoFactor: vi.fn()
}));
vi.mock('#lib/auth/session.svelte.js', () => ({
  session: {
    status: 'signed_out',
    load: vi.fn().mockResolvedValue(undefined),
    clear: vi.fn()
  }
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('renders passkey and provider actions as pill-shaped sign-in fields', async () => {
  render(Page);

  const passkey = await screen.findByRole('button', { name: 'Use a passkey' });
  await waitFor(() =>
    expect(
      screen.getByRole('link', { name: 'Continue with GitHub' })
    ).toBeTruthy()
  );

  const actions = [
    passkey,
    screen.getByRole('link', { name: 'Continue with GitHub' }),
    screen.getByRole('link', { name: 'Continue with Google' })
  ];

  for (const action of actions) {
    expect(action.className).toContain('rounded-full');
    expect(action.className).not.toContain('rounded-lg');
  }
});

// test-change: ACC-1
it('offers the demo account sign-in when the deployment advertises it', async () => {
  const { signInWithPassword } = await import('#lib/auth/security.ts');
  render(Page);

  const summary = await screen.findByText('Use the demo account');
  await fireEvent.click(summary);

  const email = screen.getByLabelText('Demo email') as HTMLInputElement;
  expect(email.value).toBe('demo@example.co');
  const password = screen.getByLabelText('Demo password');
  await fireEvent.input(password, { target: { value: 'hunter2' } });
  await fireEvent.click(
    screen.getByRole('button', { name: 'Sign in as the demo account' })
  );

  expect(signInWithPassword).toHaveBeenCalledWith(
    'demo@example.co',
    'hunter2',
    expect.any(String)
  );
});

// The login mount resolves the session to signed_out; an in-page sign-in must
// drop that cached answer before navigating or the destination loops back here.
it('clears the cached session before navigating after a successful sign-in', async () => {
  const { signInWithPassword } = await import('#lib/auth/security.ts');
  const { session } = await import('#lib/auth/session.svelte.js');
  const { goto } = await import('$app/navigation');
  vi.mocked(signInWithPassword).mockResolvedValue({
    two_factor_required: false,
    return_to: '/dashboard'
  } as never);
  render(Page);

  const summary = await screen.findByText('Use the demo account');
  await fireEvent.click(summary);
  await fireEvent.input(screen.getByLabelText('Demo password'), {
    target: { value: 'hunter2' }
  });
  await fireEvent.click(
    screen.getByRole('button', { name: 'Sign in as the demo account' })
  );

  await waitFor(() => expect(goto).toHaveBeenCalled());
  expect(session.clear).toHaveBeenCalled();
  expect(vi.mocked(session.clear).mock.invocationCallOrder[0]).toBeLessThan(
    vi.mocked(goto).mock.invocationCallOrder[0]
  );
});
