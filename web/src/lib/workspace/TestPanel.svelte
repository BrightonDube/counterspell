<script lang="ts">
  import type { TerminalPayload } from '../runs/reducer';
  import type { PublicTest, RunStatus } from '../runs/types';
  import { rowView, runOutcomeLabel, statusLabel } from './test-rows';

  let {
    status,
    tests,
    terminal
  }: {
    status: RunStatus;
    tests: readonly PublicTest[];
    terminal?: TerminalPayload;
  } = $props();

  const rows = $derived(tests.map(rowView));
</script>

<section aria-labelledby="tests-heading" class="flex flex-col gap-3">
  <h2 id="tests-heading" class="text-base font-semibold text-text">Tests</h2>
  <p class="text-sm text-text-muted">
    {terminal ? runOutcomeLabel(terminal) : statusLabel(status)}
  </p>
  {#if rows.length > 0}
    <ol class="flex flex-col gap-2">
      {#each rows as row (row.ordinal)}
        <li class="rounded border border-border p-2 text-sm text-text">
          <span>
            <span aria-hidden="true">{row.icon}</span>
            Test {row.ordinal}: {row.label}, {row.timeLabel}
          </span>
          {#if row.kind === 'sample'}
            <dl class="mt-2 grid gap-1 font-mono text-xs">
              <dt class="text-text-subtle">Input</dt>
              <dd>
                <pre class="whitespace-pre-wrap break-all">{row.input}</pre>
              </dd>
              <dt class="text-text-subtle">Expected</dt>
              <dd>
                <pre class="whitespace-pre-wrap break-all">{row.expected}</pre>
              </dd>
              <dt class="text-text-subtle">Got</dt>
              <dd>
                <pre class="whitespace-pre-wrap break-all">{row.got}</pre>
              </dd>
            </dl>
          {/if}
        </li>
      {/each}
    </ol>
  {/if}
</section>
