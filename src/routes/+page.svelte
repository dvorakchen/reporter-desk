<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import { Switch } from "$lib/components/ui/switch/";
  import { Label } from "$lib/components/ui/label/";
  import * as Table from "$lib/components/ui/table/";
  import { invoke } from "@tauri-apps/api/core";
  import { SourceNames, type NewsTitle } from "@/lib/models/newsTitle";
  import NewsWindow from "@/lib/components/news-window.svelte";
  import GenVideo from "@/lib/components/gen-video.svelte";
  import "@/app.css";
  import Config from "@/lib/components/config.svelte";
  import { config } from "@/shared";
  import Loader from "@lucide/svelte/icons/loader-2";
  import AlertBar from "@/lib/components/alert-bar/alert-bar.svelte";
  import SourceBar from "@/lib/components/source-bar.svelte";
  import { disabledAutoRefresh, enableAutoRefresh } from "./auterefresh";

  let hotNewsTitles = $state(getHotNews());
  let autoRefresh = $state(false);
  let sourceName = $state(SourceNames.PengPai);

  let configError = $derived(
    (() => {
      let msg = "";
      if (!$config.deepSeekKey) {
        msg += "需要配置 DeepSeek Api Key，";
      }
      if (!$config.aliKey) {
        msg += "需要配置 阿里百炼 Api Key";
      }
      return msg;
    })()
  );

  async function getHotNews(): Promise<NewsTitle[]> {
    return await invoke("get_hot_news_list", { source: sourceName });
  }

  async function onRefresh(e: Event) {
    hotNewsTitles = getHotNews();
  }

  $effect(() => {
    hotNewsTitles = getHotNews();
    (async () => {
      const [deepSeekKey, aliKey]: [string, string] =
        await invoke("get_config");
      $config = {
        deepSeekKey,
        aliKey,
      };
    })();
  });

  $effect(() => {
    if (autoRefresh) {
      return enableAutoRefresh(async () => {
        hotNewsTitles = getHotNews();
      });
    } else {
      disabledAutoRefresh();
    }
  });
</script>

<main class="flex flex-col p-8">
  <div class="fixed z-50 top-4 left-0 right-0 flex justify-center">
    <div class="w-fit">
      <AlertBar />
    </div>
  </div>
  <section>
    <div class="flex gap-4">
      <Button class="cursor-pointer" onclick={onRefresh}>刷新新闻热点</Button>

      <div class="flex items-center space-x-2">
        <Switch id="autoRefresh" bind:checked={autoRefresh} />
        <Label for="autoRefresh">自动刷新(30分钟)</Label>
      </div>

      <div class="text-sm text-red-500">{configError}</div>

      <div class="grow flex items-center justify-end">
        <Config />
      </div>
    </div>
  </section>

  <section>
    <SourceBar
      onchange={(selected) => {
        sourceName = selected;
      }}
    />
  </section>

  <section>
    <Table.Root>
      <Table.Caption>新闻热点</Table.Caption>
      <Table.Header>
        <Table.Row>
          <Table.Head class="max-w-2xs">标题</Table.Head>
          <Table.Head class="text-right"></Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#await hotNewsTitles}
          <div class="flex items-center justify-center gap-4 my-8">
            <span class="animate-spin">
              <Loader class="spin-in" />
            </span>
            <span class="text-lg">加载中...</span>
          </div>
        {:then hotNewsTitles}
          {#each hotNewsTitles as title}
            <Table.Row>
              <Table.Cell class="font-medium">
                <NewsWindow url={title.url} title={title.title} />
              </Table.Cell>
              <Table.Cell class="text-right flex space-x-2">
                {#if configError}
                  <Button disabled>需配置后可生成短视频</Button>
                {:else}
                  <GenVideo newsTitle={title} />
                {/if}
              </Table.Cell>
            </Table.Row>
          {/each}
        {/await}
      </Table.Body>
    </Table.Root>
  </section>
</main>
