<script setup lang="ts">

import {useTransInfoListStore} from "@/store";
import {useThemeVars} from "naive-ui";
import {TransInfo} from "@/types";
import {invoke} from "@tauri-apps/api/core";
const themeVars = useThemeVars()

const transInfoListStore = useTransInfoListStore()

const calcPercentage = function (t: TransInfo) {
  return (t.trans_size / t.total_size * 100).toFixed(1)
}

const start_trans = function (){
  invoke('start_trans').then((msg) => {
    console.log(msg)
  })
}

</script>

<template>
  <n-flex vertical class="section">
    <button class="trans-btn" @click="start_trans">发送到设备</button>
    <n-list>
      <n-list-item v-for="t in transInfoListStore.transInfos" :key="t.filename+t.to">
        <div v-if="t.trans_type == 'Send'">
          <span style="margin-left: 10px">{{t.filename}}</span><strong> ->{{t.to}}</strong>
          <n-progress :height="10" :color="themeVars.successColor" :percentage="calcPercentage(t)" :processing="!t.is_done"></n-progress>
        </div>
        <div v-else>
          <strong>{{t.from}} -></strong><span style="margin-left: 10px">{{t.filename}}</span>
          <n-progress :height="10" :color="themeVars.warningColor" :percentage="calcPercentage(t)" :processing="!t.is_done"></n-progress>
        </div>
      </n-list-item>
    </n-list>
  </n-flex>


</template>

<style scoped>
.trans-btn{
  font-size: 16px;
  color: #fff;
  background: #4361ee;
  border: none;
  border-radius: 8px;
  font-weight: 500;
  cursor: pointer;
  padding: 12px;
  transition: background 0.2s;
}

.trans-btn:hover:not(:disabled){
  background: #3a56d4;
}

</style>