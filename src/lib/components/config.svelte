<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog/";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Settings from "@lucide/svelte/icons/settings";
  import { invoke } from "@tauri-apps/api/core";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { config } from "@/shared";
  import { pushAlert } from "./alert-bar/alerts.svelte";

  let newDeepSeekKey = $state("");
  let newAliKey = $state("");
  let saved = $state(false);

  $effect(() => {
    newDeepSeekKey = $config.deepSeekKey;
    newAliKey = $config.aliKey;
  });

  async function onSave() {
    newDeepSeekKey = newDeepSeekKey.trim();
    newAliKey = newAliKey.trim();

    try {
      await invoke("set_config", {
        dsKey: newDeepSeekKey,
        aliKey: newAliKey,
      });
      $config = {
        deepSeekKey: newDeepSeekKey,
        aliKey: newAliKey,
      };
    } catch (e) {
      console.error(e);
      pushAlert('error', e as string);
    }

    saved = true;
  }
</script>

<Dialog.Root>
  <Dialog.Trigger>
    <span class="cursor-pointer flex items-center hover:underline">
      <Settings />
      配置
    </span>
  </Dialog.Trigger>
  <Dialog.Content>
    <Dialog.Description>
      <div class=" space-y-4">
        <Button
          onclick={() => {
            openPath("");
          }}>查看配置教程</Button
        >

        <div class="flex w-full max-w-sm flex-col gap-2">
          <Label for="deepseek-key">DeepSeek API Key</Label>
          <Input
            id="deepseek-key"
            placeholder="请输入你的 DeepSeek API Key"
            bind:value={newDeepSeekKey}
          />
          <Button
            class=" text-left"
            variant="link"
            onclick={() => {
              openUrl("https://platform.deepseek.com/api_keys");
            }}
            >你可以点击这里跳转注册 DeepSeek 账号并创建你的 API Key
            <br />
            充值 10 元可用 500 次
          </Button>
        </div>
        <hr />
        <div class="flex w-full max-w-sm flex-col gap-2">
          <Label for="ali-key">阿里百炼 API Key</Label>
          <Input
            id="ali-key"
            placeholder="请输入你的 阿里百炼 API Key"
            bind:value={newAliKey}
          />
          <Button
            class=" text-left"
            variant="link"
            onclick={() => {
              openUrl("https://bailian.console.aliyun.com/#/api-key");
            }}
            >你可以点击这里跳转注册阿里账号并创建你的阿里 API Key
            <br />
            新用户有 100W 免费 Token 额度</Button
          >
        </div>
        <Button onclick={onSave} type="button">
          {#if saved}
            已保存
          {:else}
            保存
          {/if}
        </Button>
      </div>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>
