<script setup lang="ts">
import {useTransFileListStore} from "@/store";
import {invoke} from "@tauri-apps/api/core";
import {open} from '@tauri-apps/plugin-dialog';

const transFileListStore = useTransFileListStore()

const openFile = async function () {
  const openFile = await open({
    multiple: true,
    directory: false,
  });

  invoke('add_file',{ files:openFile}).then((msg)=>{
    console.log(msg)
  })
  console.log(openFile)
}

const removeFile = function (filename: string) {
  invoke('remove_file', {name: filename}).then((msg)=>{
    console.log(msg)
  })
}


</script>

<template>
  <n-flex vertical class="section">
    <n-flex justify="space-between" class="section-title">
      <span>待发送文件</span>
      <span>{{transFileListStore.fileCount}} 个</span>
    </n-flex>
    <div>
      <button class="confirm-btn choose-file-btn" @click="openFile">选择文件</button>
    </div>
    <n-list>
      <n-list-item v-for="l in transFileListStore.fileList" :key="l.file">
        {{l.filename}}
        <template #suffix>
          <button class="del-btn" @click="removeFile(l.filename)">X</button>
        </template>
      </n-list-item>
    </n-list>
  </n-flex>
</template>

<style scoped>
.choose-file-btn{
  font-size: 13px;
}
</style>