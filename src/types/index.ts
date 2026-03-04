export interface CurrentClient {
    local_ip: string,
    port: number,
    name: string,
    save_path: string
}

export interface IntranetClient {
    name: string,
    ip: string,
    port: number|undefined,
    selected: boolean,
    auto_find: boolean, // 是否自动查找
}

export interface TransFile{
    filename: string,
    file: string,
    size: number,
}

export interface TransInfo{
    filename: string,
    from: string,
    to: string,
    trans_type: string, // Send|"Receive
    total_size: number,
    trans_size: number,
    is_done: boolean,
}

export interface ReqTransferData{
    filename: string,
}

export interface EmitAsk {
    uid: string,
    files: ReqTransferData[]
}