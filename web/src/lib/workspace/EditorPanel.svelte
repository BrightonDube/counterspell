<script lang="ts">
  // Editor region: a language picker limited to the languages the server enables, the lazy Monaco editor,
  // and a calm draft status line. Drafts live in the editor session, not in the component.
  import { SvelteMap } from 'svelte/reactivity';
  import MonacoEditor from '../editor/MonacoEditor.svelte';
  import type { LanguageSummary, ProblemDetail } from '../api/types';
  import {
    createEditorSession,
    type EditorSession,
    type SessionStatus
  } from './editor-session';

  let {
    problem,
    languages
  }: { problem: ProblemDetail; languages: LanguageSummary[] } = $props();

  const offered = $derived(languages.filter((language) => language.enabled));
  let chosen = $state('');
  const current = $derived(
    offered.some((language) => language.id === chosen)
      ? chosen
      : (offered[0]?.id ?? '')
  );
  const currentName = $derived(
    offered.find((language) => language.id === current)?.displayName ?? ''
  );

  const statuses = new SvelteMap<string, SessionStatus>();
  // A raw state so the session object is not wrapped in a proxy.
  let session = $state.raw<EditorSession | null>(null);

  // One session per problem. Its drafts are written when the learner pauses and when the panel is left.
  $effect(() => {
    const created = createEditorSession({
      storage: browserStorage(),
      slug: problem.slug,
      languages: offered.map((language) => ({
        id: language.id,
        maxSourceBytes: language.maxSourceBytes
      })),
      starters: problem.starters,
      debounceMs: 400,
      onStatus: (language, status) => statuses.set(language, status)
    });
    session = created;
    return () => {
      created.dispose();
      session = null;
    };
  });

  const source = $derived(session && current ? session.source(current) : '');

  const statusText = $derived.by(() => {
    switch (statuses.get(current)) {
      case 'pending':
        return 'Saving draft…';
      case 'saved':
        return 'Draft saved in this browser.';
      case 'failed':
        return 'Draft not saved. Your source stays on this page until you leave.';
      default:
        return '';
    }
  });

  /** Reads local storage safely. A browser that blocks storage gets a session that reports failed saves. */
  function browserStorage() {
    try {
      return window.localStorage;
    } catch {
      return null;
    }
  }
</script>

<section aria-label="Editor" class="flex flex-col gap-3">
  <div class="flex flex-wrap items-center gap-3">
    <label class="flex items-center gap-2 text-sm">
      <span class="text-text-muted">Language</span>
      <select
        value={current}
        onchange={(event) => (chosen = event.currentTarget.value)}
        class="min-h-11 rounded border border-border-strong bg-surface px-2 text-text"
      >
        {#each offered as language (language.id)}
          <option value={language.id}>{language.displayName}</option>
        {/each}
      </select>
    </label>
    <p aria-live="polite" class="text-sm text-text-muted">{statusText}</p>
  </div>

  {#if current && session}
    <MonacoEditor
      value={source}
      language={current}
      label={`${currentName} solution editor`}
      onChange={(next) => session?.setSource(current, next)}
    />
  {:else}
    <p class="text-sm text-text-muted">
      No language is enabled for this problem.
    </p>
  {/if}
</section>
