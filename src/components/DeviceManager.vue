<script setup lang="ts">
import {useCurrentClientStore, useIntranetClientListStore} from "@/store";
import {ref} from "vue";
import {IntranetClient} from "@/types";
import {FormInst, useThemeVars} from "naive-ui";
import {invoke} from "@tauri-apps/api/core";

const intranetClientListStore = useIntranetClientListStore()
const currentClientStore = useCurrentClientStore()
const showAddDevice = ref<boolean>(false)

const formRef = ref<FormInst | null>(null)
const deviceFrom = ref<IntranetClient>({
  name: '',
  ip: '',
  port: 0,
  selected: false,
  auto_find: false
})
const themeVars = useThemeVars()
const enableSearchBtn = ref<boolean>(false)
const searchPercentage = ref<number>(100)


const addClient = function () {
  invoke('add_client',{'client':deviceFrom.value}).then((msg)=>{
    console.log(msg)
    deviceFrom.value = {
      name: '',
      ip: '',
      port: 0,
      selected: false,
      auto_find: false
    }
    showAddDevice.value = false
  })
}

const selectedClient = function (l: IntranetClient) {
  // name: string, selected: boolean
  if (is_current_instance(l)) {
    return
  }
  invoke('selected_client',{name:l.name, selected:!l.selected}).then((msg)=>{
    console.log(msg)
  })
}

const searchClient = function (){
  searchPercentage.value = 0
  enableSearchBtn.value = true

  invoke('search_client').then((msg) => {
    console.log(msg)
  })

  const interval= setInterval(()=>{
    searchPercentage.value += 1
    if (searchPercentage.value >= 100) {
      clearInterval(interval)
      enableSearchBtn.value = false

      invoke('close_search_client').then((msg)=>{
        console.log(msg)
      })

    }
  }, 100)

}

const removeClient = function (name:string) {
  invoke('remove_client',{name}).then((msg)=>{
    console.log(msg)
  })
}

const get_host_name = function (name: String) {
  return name.replace('._lns._tcp.local.','')
}

const is_current_instance = function (l: IntranetClient){
  return l.ip == currentClientStore.ip
      && l.port == currentClientStore.port
}

</script>

<template>
  <n-flex vertical class="section">
    <n-flex justify="space-between" class="section-title">
      <span>目标客户端</span>
      <button class="common-btn" @click="showAddDevice = true">+ 添加</button>
    </n-flex>
    <n-list>
      <n-list-item v-for="l in intranetClientListStore.clientList" :key="l.name">
        <div class="file-checkbox-container">
          <label>
            <input type="checkbox" :checked="l.selected" @click="selectedClient(l)" v-if="!is_current_instance(l)">
            <span>{{ get_host_name(l.name) }} ({{l.ip}}:{{l.port}})</span>
            <span v-if="is_current_instance(l)" class="self-tag">当前实例</span>
          </label>
        </div>
        <template #suffix>
          <button v-if="!l.auto_find" class="del-btn" @click="removeClient(l.name)">X</button>
        </template>
      </n-list-item>
    </n-list>
    <div>
      <button class="common-btn" @click="searchClient" :disabled = enableSearchBtn>🔄 扫描局域网</button>
    </div>
    <n-progress v-if="searchPercentage < 100"
        type="line"
        :height="5"
        :color="{ stops: ['#E3F2FD', '#4361ee'] }"
        :percentage="searchPercentage"
        :indicator-text-color="themeVars.warningColor"
        :show-indicator="false"
        :processing="true"
    >
    </n-progress>
  </n-flex>
  <n-modal
      v-model:show="showAddDevice"
      preset="card"
      title="添加客户端"
      :mask-closable="false"
      :bordered="false"
  >
    <n-form
        ref="formRef"
        inline
        :label-width="80"
        :model="deviceFrom"
        size="medium"
    >
      <n-grid :cols="15" :x-gap="24">
        <n-form-item-gi :span="15" label="名称(唯一标识)" path="name">
          <n-input v-model:value="deviceFrom.name" placeholder="" />
        </n-form-item-gi>
        <n-form-item-gi :span="8" label="IP" path="ip">
          <n-input v-model:value="deviceFrom.ip" placeholder="" />
        </n-form-item-gi>
        <n-form-item-gi :span="7" label="端口" path="port">
          <n-input-number v-model:value="deviceFrom.port" placeholder="" />
        </n-form-item-gi>
        <n-form-item-gi :span="24">
          <button class="confirm-btn add-client-btn" @click="addClient">提交</button>
        </n-form-item-gi>
      </n-grid>
    </n-form>
  </n-modal>

</template>

<style scoped>
.file-checkbox-container{
  margin: 10px 0;
  padding: 5px;
}
.file-checkbox-container input[type="checkbox"] {
  vertical-align: middle;
}
.file-checkbox-container label{
  display: block;
  width: 100%;
}
.file-checkbox-container label span{
  margin-left: 5px;
}

.file-checkbox-container:hover{
  background: var(--hover-backgroun-color);
}
.add-client-btn{
  font-size: 15px;
}
.self-tag{
  border-radius: 3px;
  background: #ffd679;
  padding: 3px;
  font-size: 12px;
  color: #333333;
}
.n-list .n-list-item{
  padding: 0;
}
</style>