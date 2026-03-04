import {createPinia, defineStore} from "pinia";
import {CurrentClient, EmitAsk, IntranetClient, TransFile, TransInfo} from "@/types";
import {computed, ref} from "vue";

const pinia = createPinia()
export default pinia

export const useCurrentClientStore = defineStore('current-client',()=>{
    const currentClient = ref<CurrentClient>()

    function update(n_c: CurrentClient){
        currentClient.value = n_c
    }

    const ip = computed(() => currentClient.value?.local_ip)
    const port = computed(() => currentClient.value?.port)
    const save_path = computed(()=> {
        if (currentClient.value) {
            if (currentClient.value.save_path !== '') {
                return currentClient.value.save_path
            }

        }
        return './'
    })
    const name = computed(()=> currentClient.value?.name)

    return {
        currentClient,
        update,
        ip,
        port,
        save_path,
        name,
    }
})

export const useIntranetClientListStore = defineStore('intranet-client-list', ()=> {

    const clients = ref<IntranetClient[]>()

    function update(l: IntranetClient[]) {
        clients.value = l
    }

    const clientList = computed(()=> clients.value)

    return{
        clients,
        update,
        clientList,
    }
})

export const useTransFileListStore = defineStore('trans-file-list', ()=>{

    const transFiles = ref<TransFile[]>()

    function update(l: TransFile[]) {
        transFiles.value = l
    }

    const fileCount = computed(()=> {
        if (transFiles.value) {
            return transFiles.value.length
        } else {
            return 0
        }
    })

    const fileList = computed(()=> transFiles.value)

    return {
        transFiles,
        update,
        fileCount,
        fileList,
    }
})

export const useTransInfoListStore = defineStore('trans-info-list', ()=>{

    const transInfos = ref<TransInfo[]>()

    function update(l: TransInfo[]){
        transInfos.value = l
    }

    return{
        transInfos,
        update
    }
})

export const useEmitAskStore = defineStore('emit-ask',()=>{
    const ask = ref<EmitAsk>()
    const showModel = ref<boolean>()

    function update(e :EmitAsk) {
        ask.value = e
        showModel.value = true
    }

    function close() {
        showModel.value = false
    }

    const message = computed(() => {
        return "收到文件:"+ (ask.value?.files?.map(file => file.filename).join(", ") || "-");
    })

    return {
        ask,
        update,
        close,
        showModel,
        message,
    }
})