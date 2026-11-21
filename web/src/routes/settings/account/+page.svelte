<script lang="ts">
  import { goto } from '$app/navigation';
  import { browser } from '$app/env';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import { onMount, tick } from 'svelte';
  import {
    AuthApiError,
    deleteAccount,
    getConfig,
    logout,
    logoutEverywhere,
    startProviderLink
  } from '#lib/auth/client.js';
  import { confirmationMatches } from '#lib/auth/delete-account.js';
  import { errorFromQuery, linkedMessage } from '#lib/auth/messages.js';
  import {
    authProblem,
    deletePasskey,
    disableTotp,
    enableTotp,
    getSecurity,
    registerPasskey,
    startTotp,
    supportsPasskeys
  } from '#lib/auth/security.ts';
  import SecurityIcon from '#lib/auth/SecurityIcon.svelte';
  import { session } from '#lib/auth/session.svelte.js';
  import AppShell from '#lib/shell/AppShell.svelte';

  type Area =
    'account' | 'methods' | 'totp' | 'passkeys' | 'sessions' | 'delete';
  const providerNames = { github: 'GitHub', google: 'Google' } as const;
  type Provider = keyof typeof providerNames;
  const providers = Object.keys(providerNames) as Provider[];
  const dateFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium'
  });

  let search = $derived(browser ? page.url.search : '');
  let linkedNote = $derived(linkedMessage(search));
  let errorNote = $derived(errorFromQuery(search));
  let availableProviders = $state<Provider[]>([]);
  let security = $state<Awaited<ReturnType<typeof getSecurity>> | null>(null);
  let busy = $state<Area | null>(null);
  let loading = $state(true);
  let loadingSecurity = $state(false);
  let securityProblem = $state('');
  let issue = $state<{ area: Area; text: string; code?: string } | null>(null);
  let notice = $state<{ area: Area; text: string } | null>(null);
  let passkeysSupported = $state(false);

  let totpSetup = $state<{ secret: string; qr: string } | null>(null);
  let disablingTotp = $state(false);
  let code = $state('');
  let totpInput = $state<HTMLInputElement>();
  let totpAction = $state<HTMLButtonElement>();
  let recoveryHeading = $state<HTMLHeadingElement>();
  let recoveryCodes = $state<string[]>([]);
  let copiedCodes = $state(false);
  let addingPasskey = $state(false);
  let passkeyName = $state('');
  let passkeyInput = $state<HTMLInputElement>();
  let passkeyAction = $state<HTMLButtonElement>();
  let removingPasskey = $state('');
  let confirmRemovePasskey = $state<HTMLButtonElement>();
  let deleteOpen = $state(false);
  let deleteInput = $state<HTMLInputElement>();
  let deleteAction = $state<HTMLButtonElement>();
  let confirmText = $state('');

  const button = 'account-button';
  const primary = 'account-button primary';
  const fieldClass =
    'w-full rounded-md border border-accent-moss-muted bg-bg px-3 py-2.5 text-base text-text placeholder:text-text-subtle caret-accent-candle';

  onMount(() => {
    passkeysSupported = supportsPasskeys();
    void loadAccount();
  });

  function signInAgain() {
    return goto(
      resolve(
        `/login?return_to=${encodeURIComponent(resolve('/settings/account'))}`
      )
    );
  }

  async function loadAccount() {
    loading = true;
    issue = null;
    try {
      await session.load();
      if (session.status === 'signed_out') {
        await signInAgain();
        return;
      }
      if (session.user && !session.user.is_demo) void loadSecurity();
      if (session.capabilities?.link_provider) {
        void getConfig()
          .then((config) => {
            availableProviders = providers.filter(
              (provider) => config.providers[provider]
            );
          })
          .catch(() => {
            availableProviders = [];
          });
      }
    } catch (failure) {
      issue = { area: 'account', text: authProblem(failure) };
    } finally {
      loading = false;
    }
  }

  async function loadSecurity() {
    loadingSecurity = true;
    securityProblem = '';
    try {
      security = await getSecurity();
    } catch (failure) {
      if (
        failure instanceof AuthApiError &&
        failure.code === 'unauthenticated'
      ) {
        await signInAgain();
      } else {
        securityProblem = authProblem(failure);
      }
    } finally {
      loadingSecurity = false;
    }
  }

  async function run(
    area: Area,
    action: () => Promise<void>,
    focusAfter?: () => HTMLElement | undefined
  ) {
    if (busy) return;
    busy = area;
    issue = null;
    notice = null;
    try {
      await action();
    } catch (failure) {
      const code = failure instanceof AuthApiError ? failure.code : undefined;
      if (code === 'unauthenticated') {
        await signInAgain();
      } else {
        issue = { area, text: authProblem(failure), code };
        if (code === 'security_conflict') void loadSecurity();
      }
    } finally {
      busy = null;
    }
    if (!issue && focusAfter) {
      await tick();
      focusAfter()?.focus();
    }
  }

  async function linkProvider(provider: Provider) {
    const { authorize_url } = await startProviderLink(provider);
    window.location.assign(authorize_url);
  }

  async function signOut(everywhere = false, returnToAccount = false) {
    await (everywhere ? logoutEverywhere() : logout());
    session.clear();
    await (returnToAccount ? signInAgain() : goto(resolve('/login')));
  }

  async function beginTotp() {
    const { secret, uri } = await startTotp();
    let qr = '';
    try {
      const { default: createQr } = await import('qrcode-generator');
      const image = createQr(0, 'M');
      image.addData(uri);
      image.make();
      qr = image.createDataURL(5, 20);
    } catch {
      // A manual setup key is available if QR rendering fails.
    }
    totpSetup = { secret, qr };
    code = '';
    await tick();
    totpInput?.focus();
  }

  async function updateTotp() {
    if (disablingTotp) {
      await disableTotp(code.trim());
      if (security) security.totp_enabled = false;
      notice = { area: 'totp', text: 'Two-factor authentication is off.' };
      disablingTotp = false;
    } else {
      const { recovery_codes } = await enableTotp(code.trim());
      recoveryCodes = recovery_codes;
      copiedCodes = false;
      if (security) security.totp_enabled = true;
      totpSetup = null;
    }
    code = '';
  }

  async function closeTotp() {
    totpSetup = null;
    disablingTotp = false;
    code = '';
    issue = null;
    await tick();
    totpAction?.focus();
  }

  async function closeRecoveryCodes() {
    recoveryCodes = [];
    copiedCodes = false;
    await tick();
    totpAction?.focus();
  }

  async function copyRecoveryCodes() {
    await navigator.clipboard.writeText(recoveryCodes.join('\n'));
    copiedCodes = true;
  }

  async function setAddingPasskey(open: boolean) {
    addingPasskey = open;
    if (!open) passkeyName = '';
    await tick();
    if (open) passkeyInput?.focus();
    else passkeyAction?.focus();
  }

  async function addPasskey() {
    await registerPasskey(passkeyName.trim() || 'Passkey');
    notice = { area: 'passkeys', text: 'Passkey added.' };
    addingPasskey = false;
    passkeyName = '';
    await loadSecurity();
  }

  async function setRemovingPasskey(id: string, open: boolean) {
    removingPasskey = open ? id : '';
    issue = null;
    await tick();
    if (open) confirmRemovePasskey?.focus();
    else document.getElementById(`remove-passkey-${id}`)?.focus();
  }

  async function removePasskey(id: string) {
    await deletePasskey(id);
    if (security)
      security.passkeys = security.passkeys.filter((key) => key.id !== id);
    removingPasskey = '';
    notice = { area: 'passkeys', text: 'Passkey removed.' };
  }

  async function setDeleteOpen(open: boolean) {
    deleteOpen = open;
    if (!open) confirmText = '';
    await tick();
    if (open) deleteInput?.focus();
    else deleteAction?.focus();
  }

  async function removeAccount() {
    await deleteAccount(confirmText);
    session.clear();
    await goto(resolve('/'));
  }
</script>

<svelte:head>
  <title>Account · Counterspell</title>
</svelte:head>

{#snippet feedback(area: Area)}
  {#if issue?.area === area}
    <div
      class="mt-3 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm"
      data-testid="account-problem"
    >
      <p class="text-danger" role="alert">{issue.text}</p>
      {#if issue.code === 'recent_sign_in_required'}
        <button
          type="button"
          class={button}
          disabled={!!busy}
          onclick={() => run(area, () => signOut(false, true))}
        >
          Sign in again
        </button>
      {:else if area === 'totp' && totpSetup && issue.code === 'security_challenge_expired'}
        <button
          type="button"
          class={button}
          disabled={!!busy}
          onclick={() => run('totp', beginTotp)}
        >
          Restart setup
        </button>
      {/if}
    </div>
  {/if}
  {#if notice?.area === area}
    <p class="mt-3 text-sm text-success" role="status">{notice.text}</p>
  {/if}
{/snippet}

<AppShell>
  <div class="account mx-auto max-w-[54rem]">
    <header class="mb-6 sm:mb-8">
      <h1 class="text-4xl font-semibold tracking-tight sm:text-[2.5rem]">
        Account
      </h1>
      <p class="mt-1.5 text-text-muted">
        Manage how you sign in and where you stay signed in.
      </p>
    </header>

    {#if loading}
      <p class="text-text-muted" role="status">Loading your account…</p>
    {:else if session.status === 'signed_in' && session.user}
      {@const user = session.user}
      {@const can = session.capabilities ?? {}}
      {@const linkedProviders = providers.filter((provider) =>
        user.providers.includes(provider)
      )}
      {@const unlinkedProviders = availableProviders.filter(
        (provider) => !linkedProviders.includes(provider)
      )}
      <div class="identity">
        <div class="min-w-0">
          <h2
            class="text-xl font-semibold tracking-tight [overflow-wrap:anywhere]"
          >
            {user.display_name || user.handle}
          </h2>
          <p
            class="mt-1 text-sm text-accent-moss-strong [overflow-wrap:anywhere]"
          >
            @{user.handle}
          </p>
          {#if user.email}
            <p class="mt-1 text-sm text-text-muted [overflow-wrap:anywhere]">
              {user.email}
            </p>
          {/if}
        </div>
        <img
          src="/art/objects/crypt-key-128.webp"
          width="88"
          height="88"
          alt=""
        />
      </div>

      <section class="section" aria-labelledby="sign-in-methods">
        <header>
          <h2 id="sign-in-methods">Sign-in methods</h2>
          <p>Ways to access your account.</p>
        </header>
        <div class="min-w-0">
          {#if linkedProviders.length}
            <ul class="divide-y divide-border">
              {#each linkedProviders as provider (provider)}
                <li class="flex min-h-11 items-center justify-between gap-3">
                  <span class="font-medium">{providerNames[provider]}</span>
                  <span
                    class="flex items-center gap-2 text-sm text-accent-moss-strong"
                  >
                    <SecurityIcon name="check" /> Connected
                  </span>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="text-text-muted">No connected sign-in providers.</p>
          {/if}
          {#if can.link_provider && unlinkedProviders.length}
            <details class="mt-3 border-t border-border pt-1">
              <summary
                class="inline-flex min-h-11 cursor-pointer items-center gap-2 rounded-sm text-sm font-medium text-accent-moss-strong hover:text-text"
              >
                <SecurityIcon name="chevron" /> Other sign-in options
              </summary>
              <p class="mb-3 text-sm text-text-muted">
                Connect another provider to this account.
              </p>
              <div class="flex flex-wrap gap-2">
                {#each unlinkedProviders as provider (provider)}
                  <button
                    type="button"
                    class={button}
                    disabled={!!busy}
                    onclick={() => run('methods', () => linkProvider(provider))}
                  >
                    Link {providerNames[provider]}
                  </button>
                {/each}
              </div>
            </details>
          {/if}
          {#if linkedNote}<p class="mt-3 text-sm text-success" role="status">
              {linkedNote}
            </p>{/if}
          {#if errorNote}<p class="mt-3 text-sm text-danger" role="alert">
              {errorNote}
            </p>{/if}
          {@render feedback('methods')}
        </div>
      </section>

      <section class="section" aria-labelledby="security-heading">
        <header>
          <h2 id="security-heading">Security</h2>
          <p>Protect your account and manage passkeys.</p>
        </header>
        <div class="min-w-0">
          {#if user.is_demo}
            <p class="text-text-muted">
              Security settings can’t be changed on the demo account.
            </p>
          {:else if loadingSecurity && !security}
            <p class="text-text-muted" role="status">
              Loading security settings…
            </p>
          {:else if security}
            <div class="security-method">
              <div class="method-icon"><SecurityIcon name="shield" /></div>
              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
                  <h3 class="font-semibold">Authenticator app</h3>
                  <span
                    class={security.totp_enabled
                      ? 'text-accent-moss-strong text-xs'
                      : 'text-text-muted text-xs'}
                  >
                    {security.totp_enabled ? 'On' : 'Off'}
                  </span>
                </div>
                <p class="mt-1 text-sm text-text-muted">
                  Require a second code when you sign in.
                </p>
              </div>
              {#if !totpSetup && !disablingTotp && !recoveryCodes.length}
                <button
                  class={security.totp_enabled ? button : primary}
                  type="button"
                  disabled={!!busy}
                  bind:this={totpAction}
                  onclick={async () => {
                    if (security?.totp_enabled) {
                      issue = null;
                      notice = null;
                      code = '';
                      disablingTotp = true;
                      await tick();
                      totpInput?.focus();
                    } else {
                      await run('totp', beginTotp);
                    }
                  }}
                >
                  {busy === 'totp'
                    ? 'Please wait…'
                    : security.totp_enabled
                      ? 'Turn off'
                      : 'Set up'}
                </button>
              {/if}
            </div>

            {#if totpSetup || disablingTotp}
              <form
                class="inline-panel"
                onsubmit={(event) => {
                  event.preventDefault();
                  void run('totp', updateTotp, () =>
                    recoveryCodes.length ? recoveryHeading : totpAction
                  );
                }}
              >
                <h4 class="font-semibold">
                  {disablingTotp
                    ? 'Turn off two-factor authentication'
                    : 'Set up your authenticator'}
                </h4>
                {#if totpSetup}
                  {#if totpSetup.qr}
                    <p class="mt-1 text-sm text-text-muted">
                      Scan this QR code in your authenticator app.
                    </p>
                    <img
                      class="my-4 size-40 rounded-sm bg-white [image-rendering:pixelated]"
                      src={totpSetup.qr}
                      alt="Authenticator setup QR code"
                    />
                    <details class="mb-4">
                      <summary
                        class="w-fit cursor-pointer rounded-sm text-sm text-accent-moss-strong underline underline-offset-4"
                      >
                        Enter a setup key instead
                      </summary>
                      <p class="mt-2 text-sm text-text-muted">
                        In your app, choose a time-based key.
                      </p>
                      <code
                        class="mt-1 block break-all font-mono text-sm text-text"
                      >
                        {totpSetup.secret}
                      </code>
                    </details>
                  {:else}
                    <p class="mt-1 text-sm text-text-muted">
                      Add a time-based account in your app and enter this setup
                      key.
                    </p>
                    <code
                      class="my-4 block break-all font-mono text-sm text-text"
                    >
                      {totpSetup.secret}
                    </code>
                  {/if}
                {:else}
                  <p class="mt-1 text-sm text-text-muted">
                    Your recovery codes will stop working.
                  </p>
                {/if}
                <label
                  class="mb-1.5 mt-4 block text-sm"
                  for="authenticator-code"
                >
                  {disablingTotp
                    ? 'Authenticator code or recovery code'
                    : '6-digit code from your app'}
                </label>
                <input
                  id="authenticator-code"
                  class={fieldClass}
                  inputmode={disablingTotp ? 'text' : 'numeric'}
                  pattern={disablingTotp ? undefined : '[0-9]{6}'}
                  maxlength={disablingTotp ? 64 : 6}
                  autocomplete={disablingTotp ? 'off' : 'one-time-code'}
                  autocapitalize="off"
                  spellcheck={false}
                  required
                  bind:value={code}
                  bind:this={totpInput}
                />
                <div class="mt-3 flex flex-wrap gap-2">
                  <button type="submit" class={primary} disabled={!!busy}>
                    {disablingTotp
                      ? 'Turn off two-factor authentication'
                      : 'Confirm and turn on'}
                  </button>
                  <button
                    type="button"
                    class={button}
                    disabled={!!busy}
                    onclick={closeTotp}
                  >
                    Cancel
                  </button>
                </div>
              </form>
            {:else if recoveryCodes.length}
              <div class="inline-panel">
                <h4
                  class="font-semibold"
                  tabindex="-1"
                  bind:this={recoveryHeading}
                >
                  Save your recovery codes
                </h4>
                <p class="mt-1 text-sm text-text-muted">
                  Keep these somewhere safe. Each code works once if you lose
                  access to your authenticator. You’ll only see them here once.
                </p>
                <ul
                  class="my-4 grid grid-cols-1 gap-x-4 gap-y-2 font-mono text-sm sm:grid-cols-2"
                >
                  {#each recoveryCodes as recoveryCode (recoveryCode)}
                    <li><code>{recoveryCode}</code></li>
                  {/each}
                </ul>
                <div class="flex flex-wrap gap-2">
                  {#if typeof navigator !== 'undefined' && navigator.clipboard}
                    <button
                      type="button"
                      class={button}
                      disabled={!!busy}
                      onclick={() => run('totp', copyRecoveryCodes)}
                    >
                      Copy codes
                    </button>
                  {/if}
                  <button
                    type="button"
                    class={primary}
                    disabled={!!busy}
                    onclick={closeRecoveryCodes}
                  >
                    Done
                  </button>
                </div>
                {#if copiedCodes}<p
                    class="mt-2 text-sm text-success"
                    role="status"
                  >
                    Recovery codes copied.
                  </p>{/if}
              </div>
            {/if}
            {@render feedback('totp')}

            <div class="security-method mt-5 border-t border-border pt-5">
              <div class="method-icon"><SecurityIcon name="key" /></div>
              <div class="min-w-0">
                <h3 class="font-semibold">Passkeys</h3>
                <p class="mt-1 text-sm text-text-muted">
                  Sign in with your device’s screen lock or a security key.
                </p>
              </div>
              {#if passkeysSupported && !addingPasskey}
                <button
                  type="button"
                  class={button}
                  disabled={!!busy}
                  bind:this={passkeyAction}
                  onclick={() => {
                    issue = null;
                    void setAddingPasskey(true);
                  }}
                >
                  <SecurityIcon name="plus" /> Add passkey
                </button>
              {/if}
            </div>
            {#if !passkeysSupported}
              <p class="mt-3 text-sm text-text-muted">
                This browser can’t add passkeys. Try a current browser on a
                secure connection.
              </p>
            {/if}
            {#if addingPasskey}
              <form
                class="inline-panel"
                onsubmit={(event) => {
                  event.preventDefault();
                  void run('passkeys', addPasskey, () => passkeyAction);
                }}
              >
                <label
                  class="mb-1.5 block text-sm font-medium"
                  for="passkey-name"
                >
                  Passkey name <span class="text-text-muted">(optional)</span>
                </label>
                <input
                  id="passkey-name"
                  class={fieldClass}
                  autocomplete="off"
                  maxlength="64"
                  placeholder="e.g. Work laptop"
                  bind:value={passkeyName}
                  bind:this={passkeyInput}
                />
                <div class="mt-3 flex flex-wrap gap-2">
                  <button type="submit" class={primary} disabled={!!busy}>
                    {busy === 'passkeys'
                      ? 'Waiting for your device…'
                      : 'Create passkey'}
                  </button>
                  <button
                    type="button"
                    class={button}
                    disabled={!!busy}
                    onclick={() => setAddingPasskey(false)}
                  >
                    Cancel
                  </button>
                </div>
              </form>
            {/if}
            {#if security.passkeys.length}
              <ul class="mt-4 divide-y divide-border border-t border-border">
                {#each security.passkeys as passkey (passkey.id)}
                  <li
                    class="flex flex-wrap items-center justify-between gap-2 py-2.5"
                  >
                    <div class="min-w-0">
                      <p class="text-sm font-medium [overflow-wrap:anywhere]">
                        {passkey.name}
                      </p>
                      <p class="text-xs text-text-muted">
                        Added <time datetime={passkey.created_at}>
                          {dateFormat.format(new Date(passkey.created_at))}
                        </time>
                      </p>
                    </div>
                    {#if removingPasskey === passkey.id}
                      <div class="flex flex-wrap gap-2">
                        <button
                          type="button"
                          class={button}
                          disabled={!!busy}
                          bind:this={confirmRemovePasskey}
                          onclick={() =>
                            run(
                              'passkeys',
                              () => removePasskey(passkey.id),
                              () => passkeyAction
                            )}
                        >
                          Confirm removal
                        </button>
                        <button
                          type="button"
                          class={button}
                          disabled={!!busy}
                          onclick={() => setRemovingPasskey(passkey.id, false)}
                        >
                          Cancel
                        </button>
                      </div>
                    {:else}
                      <button
                        type="button"
                        class={button}
                        id={'remove-passkey-' + passkey.id}
                        aria-label={'Remove ' + passkey.name + ' passkey'}
                        disabled={!!busy}
                        onclick={() => setRemovingPasskey(passkey.id, true)}
                      >
                        Remove
                      </button>
                    {/if}
                  </li>
                {/each}
              </ul>
            {/if}
            {@render feedback('passkeys')}
          {/if}
          {#if securityProblem}
            <p class="mb-2 text-danger" role="alert">{securityProblem}</p>
            <button
              type="button"
              class={button}
              disabled={loadingSecurity}
              onclick={loadSecurity}
            >
              Reload security settings
            </button>
          {/if}
        </div>
      </section>

      <section class="section" aria-labelledby="sessions-heading">
        <header>
          <h2 id="sessions-heading">Sessions</h2>
          <p>Choose which devices to sign out.</p>
        </header>
        <div>
          <div class="flex flex-wrap gap-2.5">
            <button
              class={button}
              type="button"
              disabled={!!busy}
              onclick={() => run('sessions', () => signOut())}
            >
              <SecurityIcon name="sign-out" /> Sign out
            </button>
            {#if can.logout_everywhere}
              <button
                class={button}
                type="button"
                disabled={!!busy}
                onclick={() => run('sessions', () => signOut(true))}
              >
                Sign out everywhere
              </button>
            {/if}
          </div>
          {#if can.logout_everywhere}
            <p class="mt-2 text-sm text-text-muted">
              Signing out everywhere ends all of your sessions, including this
              one.
            </p>
          {/if}
          {@render feedback('sessions')}
        </div>
      </section>

      {#if can.delete_account}
        <section class="section" aria-labelledby="delete-heading">
          <header>
            <h2 id="delete-heading">Delete account</h2>
            <p>Permanently remove your account and submissions.</p>
          </header>
          <div>
            {#if deleteOpen}
              <form
                class="inline-panel mt-0!"
                onsubmit={(event) => {
                  event.preventDefault();
                  void run('delete', removeAccount);
                }}
              >
                <label class="mb-1.5 block text-sm" for="delete-confirm">
                  Type <code class="font-mono break-all">{user.handle}</code> to confirm
                  deletion.
                </label>
                <input
                  id="delete-confirm"
                  class={fieldClass}
                  type="text"
                  autocomplete="off"
                  bind:value={confirmText}
                  bind:this={deleteInput}
                />
                <div class="mt-3 flex flex-wrap gap-2">
                  <button
                    class={button}
                    type="submit"
                    disabled={!!busy ||
                      !confirmationMatches(confirmText, user.handle)}
                  >
                    Delete my account
                  </button>
                  <button
                    class={button}
                    type="button"
                    disabled={!!busy}
                    onclick={() => setDeleteOpen(false)}
                  >
                    Cancel
                  </button>
                </div>
              </form>
            {:else}
              <button
                class={button}
                type="button"
                disabled={!!busy}
                bind:this={deleteAction}
                onclick={() => setDeleteOpen(true)}
              >
                <SecurityIcon name="trash" /> Delete account
              </button>
            {/if}
            {@render feedback('delete')}
          </div>
        </section>
      {/if}
    {:else}
      {@render feedback('account')}
      {#if issue?.area === 'account'}
        <button type="button" class={button} onclick={loadAccount}>
          Reload account
        </button>
      {/if}
    {/if}
  </div>
</AppShell>

<style>
  .account {
    padding-block: clamp(0.75rem, 3vw, 2.5rem) 2rem;
  }

  .identity {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: clamp(1rem, 3vw, 1.5rem) clamp(1rem, 4vw, 1.75rem);
    background: linear-gradient(
      110deg,
      var(--surface) 50%,
      var(--accent-moss-deep)
    );
    border: 1px solid var(--border-strong);
    border-radius: 0.75rem;
  }

  .identity img {
    flex: none;
    inline-size: clamp(3.25rem, 10vw, 5.5rem);
    block-size: auto;
  }

  .section {
    display: grid;
    gap: 0.875rem;
    padding-block: 1.5rem;
    border-block-end: 1px solid var(--border);
  }

  .section h2 {
    font-size: 1.125rem;
    font-weight: 600;
    letter-spacing: -0.015em;
  }

  .section header p {
    margin-block-start: 0.25rem;
    max-inline-size: 25ch;
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .security-method {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: start;
    column-gap: 0.75rem;
    row-gap: 0.625rem;
  }

  .security-method > button {
    grid-column: 2;
    justify-self: start;
  }

  .method-icon {
    display: grid;
    place-items: center;
    inline-size: 2.375rem;
    block-size: 2.375rem;
    border-radius: 0.5rem;
    background: var(--raised);
    color: var(--accent-moss-strong);
  }

  .inline-panel {
    margin-block-start: 0.875rem;
    padding: 1rem;
    border: 1px solid var(--border-strong);
    border-radius: 0.5rem;
    background: var(--surface);
  }

  .account-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.375rem;
    min-block-size: 2.75rem;
    border: 1px solid var(--border-strong);
    border-radius: 0.375rem;
    padding: 0.375rem 0.8rem;
    background: var(--surface);
    color: var(--text);
    font-size: 0.875rem;
    font-weight: 550;
    cursor: pointer;
  }

  .account-button:hover:not(:disabled) {
    border-color: var(--accent-moss-muted);
    background: var(--raised);
  }

  .account-button.primary {
    border-color: var(--accent-candle);
    background: var(--accent-candle);
    color: var(--bg);
  }

  .account-button.primary:hover:not(:disabled) {
    border-color: var(--accent-candle-glow);
    background: var(--accent-candle-glow);
  }

  .account-button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  details[open] summary :global(svg) {
    transform: rotate(90deg);
  }

  @media (min-width: 48rem) {
    .section {
      grid-template-columns: minmax(10rem, 15rem) minmax(0, 1fr);
      gap: 1.5rem;
      padding-inline: 0.25rem;
    }

    .security-method {
      grid-template-columns: auto minmax(0, 1fr) auto;
      align-items: center;
    }

    .security-method > button {
      grid-column: auto;
    }
  }
</style>
