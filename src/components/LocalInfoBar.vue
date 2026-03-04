<script setup lang="ts">
import {useCurrentClientStore} from "@/store";
import {invoke} from "@tauri-apps/api/core";
import {open} from '@tauri-apps/plugin-dialog';


const currentClientStore = useCurrentClientStore()


const selectSavePath = async function (){
  const selectDir = await open({
    directory: true,
  });

  if (selectDir != null) {
    invoke('pick_selected_path',{saveDir:selectDir}).then((msg)=>{
      console.log(msg)
    })

    console.log(selectDir)
  }
}

</script>
<template>
  <n-flex vertical class="section">
    <n-flex justify="space-between">
      <span class="info">
        <span>📡 本机地址</span>
        <span>{{ currentClientStore.ip }}:{{currentClientStore.port}}</span>
      </span>
<!--      <button class="common-btn">修改</button>-->
    </n-flex>
    <n-flex justify="space-between">
      <span class="info">
        <span>📁 接收文件夹</span>
        <span>{{currentClientStore.save_path}}</span>
      </span>
      <button class="common-btn" @click="selectSavePath">修改</button>
    </n-flex>
  </n-flex>
</template>

<style scoped>
.info>*{
  margin-right: 10px;
}
</style>