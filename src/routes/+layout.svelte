<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { theme, fontSize } from '$lib/theme';

  let { children } = $props();

  onMount(() => {
    theme.init();
    fontSize.init();
  });
</script>

{@render children()}
