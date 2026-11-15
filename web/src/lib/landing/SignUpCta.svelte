<script lang="ts">
	import { CTA } from './copy';

	// Both links go to /login. Sign-up happens there through GitHub, Google or email (auth spec).
</script>

<div class="flex flex-wrap items-center gap-6">
	<a
		href="/login"
		class="cta inline-flex items-center rounded bg-accent-candle px-6 py-3 text-base font-semibold text-bg transition-colors duration-200 hover:bg-accent-candle-glow focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:ring-offset-bg focus-visible:outline-none"
	>
		{CTA.primary}
	</a>
	<a
		href="/login"
		class="text-base text-accent-moss underline-offset-4 hover:text-accent-moss-strong hover:underline focus-visible:ring-2 focus-visible:ring-focus focus-visible:outline-none"
	>
		{CTA.secondary}
	</a>
</div>
