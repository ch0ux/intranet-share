<script setup lang="ts">

import LocalInfoBar from "@/components/LocalInfoBar.vue";
import DeviceManager from "@/components/DeviceManager.vue";
import FileSelector from "@/components/FileSelector.vue";
import TransferSection from "@/components/TransferSection.vue";
import {invoke} from "@tauri-apps/api/core";
import {CurrentClient, EmitAsk, IntranetClient, TransFile, TransInfo} from "@/types";
import {listen} from "@tauri-apps/api/event";
import {onMounted, ref} from "vue";
import {
  useCurrentClientStore,
  useEmitAskStore,
  useIntranetClientListStore,
  useTransFileListStore,
  useTransInfoListStore
} from "@/store";

const currentClientStore = useCurrentClientStore()
const intranetClientListStore = useIntranetClientListStore()
const transFileListStore = useTransFileListStore()
const transInfoListStore = useTransInfoListStore()
const emitAskStore = useEmitAskStore()
const showSpin = ref<boolean>(true)

listen<CurrentClient>('e_current_client',(event) =>{
  currentClientStore.update(event.payload)
})

listen<TransFile[]>('e_files',(event) =>{
  transFileListStore.update(event.payload)
})

listen<IntranetClient[]>('e_intranet_clients',(event)=>{
  intranetClientListStore.update(event.payload)
})

listen<TransInfo[]>('e_trans', (event) =>{
  transInfoListStore.update(event.payload)
})

listen<EmitAsk>('e_ask',(event) =>{
  emitAskStore.update(event.payload)
})

const answer_accept = function (answer: boolean) {
  invoke('answer_accept', {uid:emitAskStore.ask?.uid, answer}).then(()=>{
    console.log(emitAskStore)
    emitAskStore.close()
  })
}

onMounted(()=>{
  invoke<CurrentClient>('init').then((data)=>{
    currentClientStore.currentClient = data
    showSpin.value = false
  })
})
</script>

<template>
  <n-spin :show="showSpin">
    <template #description>
      启动中...
    </template>
    <n-flex vertical>
      <LocalInfoBar></LocalInfoBar>
      <FileSelector></FileSelector>
      <DeviceManager></DeviceManager>
      <TransferSection></TransferSection>
    </n-flex>
    <n-modal
        v-model:show="emitAskStore.showModel"
        preset="dialog"
        title="接收文件"
        :content="emitAskStore.message"
        positive-text="接收"
        negative-text="拒绝"
        @positive-click="answer_accept(true)"
        @negative-click="answer_accept(false)"
    />
  </n-spin>
</template>

<style scoped>

</style>