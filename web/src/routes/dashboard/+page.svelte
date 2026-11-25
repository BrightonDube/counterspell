<script lang="ts">
  import DashboardScreen from '../../lib/dashboard/DashboardScreen.svelte';
  import AppShell from '../../lib/shell/AppShell.svelte';
  import Seo from '../../lib/seo/Seo.svelte';
</script>

<Seo
  title="Dashboard · Counterspell"
  description="Your practice at a glance — solved problems, streaks, reviews and next steps."
  path="/dashboard"
  social={false}
/>

<AppShell>
  <DashboardScreen />
</AppShell>
