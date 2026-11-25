<script lang="ts">
  import { patchProfile } from '#lib/auth/client.js';
  import {
    COUNTRIES,
    countryName,
    resolveCountry
  } from '#lib/auth/countries.js';
  import { authProblem } from '#lib/auth/security.ts';
  import { session } from '#lib/auth/session.svelte.js';

  let {
    submitLabel = 'Save profile',
    onSaved = () => {}
  }: {
    submitLabel?: string;
    onSaved?: (user: Record<string, unknown>) => void;
  } = $props();

  // The stored country code is shown as its readable name; unknown or
  // legacy codes are shown verbatim and sent back untouched when the
  // picker input is left unchanged. The baseline refreshes after every
  // successful save so cancel discards only unsaved changes.
  let initialName = String(session.user?.display_name ?? '');
  let initialCode = String(session.user?.country ?? '');
  let initialCountry = initialCode ? countryName(initialCode) : '';

  let displayName = $state(initialName);
  let country = $state(initialCountry);
  let saving = $state(false);
  let problem = $state('');
  let saved = $state(false);

  function cancel() {
    displayName = initialName;
    country = initialCountry;
    problem = '';
    saved = false;
  }

  async function submit() {
    if (saving) return;
    const name = displayName.trim();
    if (!name || name.length > 80) {
      problem = 'Enter a full name between 1 and 80 characters.';
      saved = false;
      return;
    }
    // Unchanged input round-trips the stored value verbatim so legacy
    // codes are never silently rewritten.
    const resolved =
      country === initialCountry
        ? initialCode || null
        : resolveCountry(country);
    if (resolved === undefined) {
      problem = 'Choose a country from the list, or clear the field.';
      saved = false;
      return;
    }
    saving = true;
    problem = '';
    saved = false;
    try {
      const view = await patchProfile({
        display_name: name,
        country: resolved
      });
      session.user = view.user;
      session.capabilities = view.capabilities;
      session.status = 'signed_in';
      initialName = String(view.user?.display_name ?? '');
      initialCode = String(view.user?.country ?? '');
      initialCountry = initialCode ? countryName(initialCode) : '';
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
  <label for="profile-name">Full name</label>
  <input
    id="profile-name"
    name="display_name"
    bind:value={displayName}
    maxlength="80"
    autocomplete="name"
    required
  />

  <label for="profile-country">Country</label>
  <input
    id="profile-country"
    name="country"
    bind:value={country}
    list="profile-countries"
    autocomplete="country-name"
    placeholder="Start typing a country"
  />
  <datalist id="profile-countries">
    {#each COUNTRIES as entry (entry.code)}
      <option value={entry.name}>{entry.code}</option>
    {/each}
  </datalist>

  <p class="profile-note" id="profile-country-help">
    Optional. Search by name — stored as a two-letter code.
  </p>

  <div class="profile-actions">
    <button type="submit" disabled={saving || !displayName.trim()}>
      {saving ? 'Saving…' : submitLabel}
    </button>
    <button type="button" class="ghost" disabled={saving} onclick={cancel}>
      Cancel
    </button>
  </div>

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

  .profile-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
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

  button.ghost {
    background: transparent;
    color: var(--text-muted);
  }

  button.ghost:hover:not(:disabled) {
    color: var(--text);
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
