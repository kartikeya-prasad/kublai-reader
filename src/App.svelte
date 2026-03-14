<script lang="ts">
  import { onMount } from "svelte";
  import AppShell from "./lib/components/layout/AppShell.svelte";
  import { initTheme } from "./lib/utils/theme";

  onMount(async () => {
    await initTheme();
  });
</script>

<AppShell />
