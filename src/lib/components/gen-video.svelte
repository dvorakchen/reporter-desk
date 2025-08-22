<script lang="ts">
  import Loader2Icon from "@lucide/svelte/icons/loader-2";
  import Button from "$lib/components/ui/button/button.svelte";
  import type { NewsTitle } from "../models/newsTitle";
  import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { invoke } from "@tauri-apps/api/core";
  import { pushAlert } from "./alert-bar/alerts.svelte";

  const { newsTitle }: { newsTitle: NewsTitle } = $props();
  let loading = $state(false);
  let path = $state<string | null>(null);

  async function onGenerateVideo(_: Event) {
    loading = true;

    try {
      const [deepseek_key, ali_key]: [string, string] =
        await invoke("get_config");
      if (!deepseek_key || !ali_key) {
        return;
      }

      path = await invoke("gen_video", { newsTitle: newsTitle });
    } catch (error) {
      console.error("Error generating video:", error);
      pushAlert("生成失败，请重试", error as string);
    } finally {
      loading = false;
    }
  }

  function onOpenFolder() {
    if (path) {
      revealItemInDir(path);
    } else {
      console.warn("No path to open");
    }
  }
</script>

<div>
  <Button onclick={onGenerateVideo} disabled={loading}>
    {#if loading}
      <Loader2Icon class="animate-spin" />
    {/if}
    生成短视频</Button
  >
  {#if path}
    <Button onclick={() => openPath(path!)}>打开视频</Button>
    <Button onclick={onOpenFolder}>打开文件夹</Button>
  {/if}
</div>
