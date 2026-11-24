<script lang="ts">
  // AQ01: shared head metadata for public and authenticated routes. Canonical and
  // social URLs always point at the production origin, whatever host serves the
  // page (preview, staging, local test server), because they name the canonical
  // home of the route, not the request's host.
  const SITE = 'https://counterspel.online';
  const DEFAULT_IMAGE = '/art/landing/moon-graveyard-poster-1920.webp';

  let {
    title,
    description,
    path,
    image = DEFAULT_IMAGE,
    social = true
  }: {
    title: string;
    description: string;
    /** The route path, e.g. `/problems`. */
    path: string;
    /** Social-card image path on the site origin. */
    image?: string;
    /** Set false on routes that should not be shared (account screens). */
    social?: boolean;
  } = $props();

  const url = $derived(`${SITE}${path}`);
  const imageUrl = $derived(
    image.startsWith('http') ? image : `${SITE}${image}`
  );
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="description" content={description} />
  <link rel="canonical" href={url} />
  {#if social}
    <meta property="og:title" content={title} />
    <meta property="og:description" content={description} />
    <meta property="og:url" content={url} />
    <meta property="og:type" content="website" />
    <meta property="og:site_name" content="Counterspell" />
    <meta property="og:image" content={imageUrl} />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content={title} />
    <meta name="twitter:description" content={description} />
    <meta name="twitter:image" content={imageUrl} />
  {/if}
</svelte:head>
