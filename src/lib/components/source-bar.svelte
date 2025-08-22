<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import { SourceNames, sourceNameToDisplayName } from "../models/newsTitle";

  const {
    onchange = null,
  }: {
    onchange: ((sourceName: string) => void) | null;
  } = $props();

  const sourceNames = Object.values(SourceNames);
  let selected = $state(SourceNames.PengPai);

  function roundedStyle(i: number) {
    if (sourceNames.length === 1) {
      return "";
    }

    if (i === 0) return "rounded-r-none";
    if (i === sourceNames.length - 1) return "rounded-l-none";
    return "rounded-none";
  }

  function onClick(sourceName: string) {
    selected = sourceName;

    if (onchange !== null) onchange(sourceName);
  }
</script>

<div class="flex my-4">
  {#each sourceNames as sourceName, i}
    <Button
      class="cursor-pointer 
    {roundedStyle(i)}
    "
      variant={sourceName === selected ? "default" : "outline"}
      onclick={() => onClick(sourceName)}
    >
      {sourceNameToDisplayName(sourceName)}
    </Button>
  {/each}
</div>
