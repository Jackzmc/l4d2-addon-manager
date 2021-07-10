<template>
<div>
    <div class="box" v-if="updating">
      <div v-for="(update,key) in updates" :key="key">
        <b>{{update.title}}</b> <em>({{key}})</em><br>
        <div style="width:60%" class="is-inline">
          <progress class="progress is-success is-inline-block" :value="update.bytes_downloaded" :max="update.bytes_total" style="width:60%" >
            {{update.bytes_downloaded / update.bytes_total * 100}}%
          </progress>
        </div>
        <div class="is-inline" style="margin-left: 1em">
          <p v-if="update.error" class="has-text-danger">
            <b>Failed: </b>{{update.error}}
          </p>
          <template v-else-if="update.complete">
            Complete 
          </template>
          <template v-else>
            {{formatBytes(update.bytes_downloaded)}} / {{formatBytes(update.bytes_total)}}
            &nbsp; ({{Math.round(update.bytes_downloaded / update.bytes_total * 100)}}%)
          </template>
        </div>
      </div>
    </div>
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <th style="width: 40px">
                    <b-checkbox @input="onSelectAll" />
                </th>
                <th>Item Name</th>
                <th>File Size</th>
                <th>Last Updated</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="item in items" :key="item.publishedfileid" >
                <td><b-checkbox v-model="selected[item.publishedfileid]" /></td>
                <td @click="selected[item.publishedfileid] = !selected[item.publishedfileid]">
                    <a 
                        target="_blank" 
                        :href="'https://steamcommunity.com/sharedfiles/filedetails/?id=' + item.publishedfileid"
                        class="has-text-info"
                    >
                        {{item.title || item.publishedfileid}}
                    </a>
                </td>
                <td>{{formatBytes(item.file_size)}}</td>
                <td>{{formatDate(item.time_updated)}}</td>
            </tr>
        </tbody>
        <tfoot>
            <tr>
                <td></td>
                <th>Total File Size: </th>
                <th>{{formatBytes(total_bytes)}}</th>
                <td></td>
            </tr>
        </tfoot>
    </table>
    <template v-if="hasItemSelected">
        <hr>
        <div class="container ml-5" v-if="hasItemSelected">
            <b>Action for selected items</b><br>
            <div class="buttons">
                <b-button class="button is-info" @click="update" :disabled="updating" :loading="updating">Update</b-button>
                <b-button class="button is-danger" @click="markUpdated" :loading="updating">Mark as Updated</b-button>
                <!-- <a class="button is-success">Enable</a>
                <a class="button is-danger">Disable</a> -->
            </div>
        </div>
    </template>
</div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import Fuse from 'fuse.js'
import { formatBytes, formatDate } from '@/js/utils'

const CONCURRENT_DOWNLOADS = 3

export default {
    props: ['items'],
    data() {
        return {
            active: false,
            selected: {},
            updates: {},
            updating: false,
            search: {
                active: false,
                value: ""
            }
        }
    },
    computed: {
        total_bytes() {
            let bytes = 0;
            for(const item in this.items) {
                bytes += this.items[item].file_size
            }
            return bytes
        },
        hasItemSelected() {
           for(const item in this.selected) {
               if(this.selected[item] === true) return true
           } 
           return false;
        },
        itemsFiltered() {
            if(this.search.value === "") return this.items
            const fuse = new Fuse(this.items, {
                keys: ['title', 'author'],
                distance: 15,
                threshold: 0.5,
                includeScore: true
            })
            return fuse.search(this.search.value).map(r => r.item)
        }
    },
    methods: {
        formatBytes, 
        formatDate,
        async update() {
            let items = this.items.filter(item => this.selected[item.publishedfileid])
            if(items.length == 0) return
            this.updating = true;
            items.forEach(item => {
                this.$set(this.updates, item.publishedfileid, {
                    bytes_total: item.file_size,
                    bytes_downloaded: 0,
                    complete: false,
                    title: item.title
                })
                //TODO: Add back
                this.selected[item.publishedfileid] = false;
            })
            let running = 0;
            let timer = setInterval(async() => {
                if(items.length == 0 && running == 0) {
                    this.$buefy.toast.open({
                        message: 'All addons were updated successfully',
                        type: 'is-success'
                    })
                    this.$emit('refreshItems')
                    this.updating = false
                    for(const item in items) {
                        this.$delete(this.updates, item.publishedfileid)
                    }
                    return clearInterval(timer)
                }else if(running < CONCURRENT_DOWNLOADS) {
                    let item = items.shift();
                    running++
                    await invoke("download_addon", { item })
                    .catch(err => this.updates[item.publishedfileid].error = err)
                    running--
                }
            }, 1000)
        },
        onSelectAll(state) {
            for(const item of this.items) {
                this.$set(this.selected, item.publishedfileid, state)
            }
        },
        markUpdated() {
            const selected = [];
            for(const item of this.items) {
                if(this.selected[item.publishedfileid]) {
                    selected.push(item)
                }
            }
            this.$buefy.dialog.confirm({
                message: `<h5 class='title is-5'>Are you sure you want to mark the following addons as updated?</h5>
                This will not update the addons, this simply marks them as on the latest version, incase they were updated externally.<br>
                <div class="content">
                <ul>
                    ${selected.map(item => `<li>${item.title}</li>`).join("")}
                </ul>
                </div>`,
                confirmText: 'Mark as Updated',
                onConfirm: async() => {
                    try {
                        const updated = await invoke('mark_addons_updated', { items: selected })
                        if(updated == selected.length) {
                            this.$buefy.toast.open({
                                message: 'All addons marked successfully',
                                type: 'is-success'
                            })
                        }else{
                            this.$buefy.toast.open({
                                message: `${selected.length - updated} addons failed to be marked`,
                                type: 'is-warning'
                            })
                        }
                        this.$emit('refreshItems')
                    } catch (err) {
                        console.log(err)
                        this.$buefy.dialog.alert({
                            message: 'An error occurred while marking addons:<br>' + err.message,
                            type: 'is-danger',
                            ariaRole: 'alertdialog',
                            ariaModal: true
                        })
                    }
                }
            })
        }
    },
    async created() {
        await listen('progress', ({payload}) => {
        if(payload.error) {
            return console.error(`${payload.publishedfileid} -> ${payload.error}`)
        }
        this.updates[payload.publishedfileid] = {
            ...this.updates[payload.publishedfileid],
            bytes_downloaded: payload.bytes_downloaded,
            complete: payload.complete
        }
        if(payload.complete) {
            setTimeout(() => this.$delete(this.updates, payload.publishedfileid), 5000)
        }
        })
    },
}
</script>