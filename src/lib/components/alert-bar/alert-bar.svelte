<script lang="ts">
  import * as Alert from "$lib/components/ui/alert";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import { alerts } from "./alerts.svelte";

  const { type = "" } = $props();
</script>

{#each alerts as alert}
  <Alert.Root variant="destructive">
    <AlertCircleIcon />
    <Alert.Title>{alert.title}</Alert.Title>
    <Alert.Description>
      <p>{alert.desc}</p>
    </Alert.Description>
  </Alert.Root>
{/each}
