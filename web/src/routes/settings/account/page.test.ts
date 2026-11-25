import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor
} from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Page from './+page.svelte';

vi.mock('$app/env', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/settings/account') }
}));
vi.mock('#lib/auth/client.js', () => ({
  AuthApiError: class extends Error {
    constructor(
      public status: number,
      public code = 'unauthenticated'
    ) {
      super(code);
    }
  },
  deleteAccount: vi.fn(),
  getConfig: vi.fn().mockResolvedValue({
    providers: { github: true, google: true }
  }),
  logout: vi.fn(),
  logoutEverywhere: vi.fn(),
  patchProfile: vi.fn().mockResolvedValue({
    user: {
      handle: 'witch-hunter',
      display_name: 'Night Walker',
      email: 'witch@example.com',
      country: 'ZA',
      profile_complete: true,
      avatar_url: '/art/creatures/skull-green-eyes.webp',
      avatar_editable: false,
      providers: ['github'],
      is_demo: false
    },
    capabilities: { edit_profile: true }
  }),
  startProviderLink: vi.fn().mockResolvedValue(undefined)
}));
vi.mock('#lib/auth/delete-account.js', () => ({
  confirmationMatches: vi.fn(() => false)
}));
vi.mock('#lib/auth/messages.js', () => ({
  errorFromQuery: vi.fn(() => null),
  linkedMessage: vi.fn(() => null)
}));
vi.mock('#lib/auth/security.ts', () => ({
  authProblem: vi.fn(() => 'Account unavailable.'),
  deletePasskey: vi.fn(),
  disableTotp: vi.fn(),
  enableTotp: vi.fn(),
  getSecurity: vi.fn().mockResolvedValue({
    totp_enabled: false,
    can_replace_authenticator: false,
    passkeys: []
  }),
  registerPasskey: vi.fn(),
  startTotp: vi.fn(),
  supportsPasskeys: vi.fn(() => true)
}));
vi.mock('#lib/auth/session.svelte.js', () => ({
  session: {
    status: 'signed_in',
    user: {
      handle: 'witch-hunter',
      display_name: 'Witch Hunter',
      email: 'witch@example.com',
      providers: ['github'],
      country: 'ZA',
      profile_complete: true,
      avatar_url: '/art/creatures/skull-green-eyes.webp',
      avatar_editable: false,
      is_demo: false
    },
    capabilities: {
      link_provider: true,
      logout_everywhere: true,
      delete_account: true,
      edit_profile: true
    },
    load: vi.fn().mockResolvedValue(undefined)
  }
}));

describe('settings account costume polish', () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  // Breaks if the key loses its animation hook or if the account sections are reordered.
  it('animates the crypt key while preserving the account section order', async () => {
    const { container } = render(Page);

    await waitFor(() => expect(screen.getByText('Witch Hunter')).toBeTruthy());

    const key = container.querySelector<HTMLImageElement>(
      'img.crypt-key[data-testid="account-crypt-key"]'
    );
    expect(key).not.toBeNull();
    expect(key?.getAttribute('src')).toBe('/art/objects/crypt-key-128.webp');
    expect(key?.getAttribute('alt')).toBe('');
    expect(key?.getAttribute('tabindex')).toBe('-1');
    expect(key?.getAttribute('aria-hidden')).toBe('true');

    expect(
      Array.from(container.querySelectorAll('section h2')).map(
        (heading) => heading.textContent
      )
    ).toEqual(['Sign-in methods', 'Security', 'Sessions', 'Delete account']);

    const decoration = container.querySelector<HTMLImageElement>(
      'img[src="/art/objects/cursed-crt.webp"]'
    );
    expect(decoration?.getAttribute('loading')).toBe('lazy');

    const avatar = container.querySelector<HTMLImageElement>(
      'img.profile-avatar[src="/art/creatures/skull-green-eyes.webp"]'
    );
    expect(avatar).not.toBeNull();
    expect(screen.getByRole('button', { name: 'Edit profile' })).toBeTruthy();
  });

  // The signed-in profile must not expose internal storage configuration.
  it('keeps avatar storage implementation details out of the profile', async () => {
    render(Page);

    await screen.findByRole('heading', { name: 'Witch Hunter' });
    expect(screen.queryByText(/storage is configured/i)).toBeNull();
  });

  // CPX-2: name is the primary identity heading, @handle is distinct,
  // the country renders as a readable name, and email lives once in
  // private account details — not in the identity block.
  it('presents name-first identity with a readable country', async () => {
    const { container } = render(Page);

    await screen.findByRole('heading', { name: 'Witch Hunter' });
    const identityBlock = container.querySelector('.profile-identity');
    expect(identityBlock).not.toBeNull();
    expect(identityBlock?.textContent).toContain('@witch-hunter');
    expect(identityBlock?.textContent).toContain('South Africa');
    expect(identityBlock?.textContent).not.toContain('witch@example.com');
    // Avatar is circular and sits above the text (first child on mobile).
    const avatar = identityBlock?.querySelector('img.profile-avatar');
    expect(avatar?.getAttribute('src')).toBe(
      '/art/creatures/skull-green-eyes.webp'
    );

    // Email appears exactly once, inside private account details.
    const details = container.querySelector('.account-details');
    expect(details?.textContent).toContain('witch@example.com');
    expect(container.textContent?.match(/witch@example\.com/g)).toHaveLength(1);
  });

  // CPX-2: an unset country is omitted rather than showing a raw code.
  it('omits the country row when no country is set', async () => {
    const { session } = await import('#lib/auth/session.svelte.js');
    (session.user as Record<string, unknown>).country = null;
    const { container } = render(Page);

    await screen.findByRole('heading', { name: 'Witch Hunter' });
    const identityBlock = container.querySelector('.profile-identity');
    expect(identityBlock?.textContent).not.toContain('South Africa');
    expect(identityBlock?.textContent).not.toContain('Country');
    (session.user as Record<string, unknown>).country = 'ZA';
  });

  // Breaks if linking another provider loses its visible key affordance or accessible name.
  it('gives an unlinked provider a bounded key/unlock activation affordance', async () => {
    render(Page);

    const details = await screen.findByText('Other sign-in options');
    await fireEvent.click(details);
    const link = await screen.findByRole('button', { name: 'Link Google' });

    expect(link.className).toContain('provider-link');
    expect(link.querySelector('svg')).not.toBeNull();
    expect(link.getAttribute('aria-describedby')).toBe('link-google-help');
    expect(screen.getByText('Link Google to this account.')).toBeTruthy();
  });
});
