<script lang="ts">
  import { patchProfile } from '#lib/auth/client.js';
  import { authProblem } from '#lib/auth/security.ts';
  import { session } from '#lib/auth/session.svelte.js';

  let {
    submitLabel = 'Save profile',
    onSaved = () => {}
  }: {
    submitLabel?: string;
    onSaved?: (user: Record<string, unknown>) => void;
  } = $props();

  let displayName = $state(String(session.user?.display_name ?? ''));
  let country = $state(String(session.user?.country ?? ''));
  let saving = $state(false);
  let problem = $state('');
  let saved = $state(false);

  async function submit() {
    if (saving) return;
    saving = true;
    problem = '';
    saved = false;
    try {
      const view = await patchProfile({
        display_name: displayName,
        country: country || null
      });
      session.user = view.user;
      session.capabilities = view.capabilities;
      session.status = 'signed_in';
      saved = true;
      onSaved(view.user);
    } catch (failure) {
      problem = authProblem(failure);
    } finally {
      saving = false;
    }
  }
</script>

<form
  class="profile-form"
  onsubmit={(event) => {
    event.preventDefault();
    void submit();
  }}
>
  <label for="profile-name">Display name</label>
  <input
    id="profile-name"
    name="display_name"
    bind:value={displayName}
    maxlength="80"
    autocomplete="name"
    required
  />

  <label for="profile-country">Country code</label>
  <input
    id="profile-country"
    name="country"
    bind:value={country}
    maxlength="2"
    autocomplete="country"
    placeholder="ZA"
  />

  <p class="profile-note" id="profile-country-help">
    Optional. Use a two-letter code such as ZA.
  </p>

  <button type="submit" disabled={saving || !displayName.trim()}>
    {saving ? 'Saving…' : submitLabel}
  </button>

  {#if saved}
    <p class="profile-status" role="status">Profile saved.</p>
  {/if}
  {#if problem}
    <p class="profile-problem" role="alert">{problem}</p>
  {/if}
</form>

<style>
  .profile-form {
    display: grid;
    gap: 0.625rem;
  }

  label {
    font-size: 0.875rem;
    font-weight: 500;
  }

  input {
    min-block-size: 2.75rem;
    border: 1px solid var(--accent-moss-muted);
    border-radius: 0.375rem;
    background: var(--bg);
    color: var(--text);
    caret-color: var(--accent-candle);
    padding-inline: 0.75rem;
    font: inherit;
  }

  input:focus-visible,
  button:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: 2px;
  }

  .profile-note {
    margin: 0;
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  button {
    min-block-size: 2.75rem;
    border: 1px solid var(--border-strong);
    border-radius: 0.375rem;
    background: var(--accent-moss);
    color: var(--text);
    padding-inline: 1rem;
    font: inherit;
    font-weight: 600;
  }

  button:disabled {
    cursor: not-allowed;
    border-color: var(--border);
    background: var(--surface);
    color: var(--text-muted);
  }

  .profile-status {
    margin: 0;
    color: var(--success);
    font-size: 0.875rem;
  }

  .profile-problem {
    margin: 0;
    color: var(--danger);
    font-size: 0.875rem;
  }
</style>
