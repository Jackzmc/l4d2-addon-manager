<template>
<div>
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <th style="width: 40px">
                    <b-checkbox @input="onSelectAll" />
                </th>
                <th>
                    <span>Item Name</span>
                    <div class="is-inline is-pulled-right">
                        <b-input v-model.lazy="search.value" 
                            placeholder="Search..." 
                            icon="search"
                            rounded 
                            @blur="search.active = false"
                            @focus="search.active = true"
                            :size="search.active === false ? 'is-small' : ''"
                        />
                    </div>
                </th>
                <th>File Size</th>
                <th>Last Updated</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="item in itemsFiltered" :key="item.publishedfileid" >
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
                <b-button type="is-info" 
                    @click="importAddons"
                    :disabled="loading"
                    :loading="loading"
                >
                    Import Addons
                </b-button>
            </div>
        </div>
    </template>
</div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'
import { formatBytes, formatDate } from '@/js/utils'
import Fuse from 'fuse.js'

export default {
    props: ['items'],
    data() {
        return {
            active: false,
            selected: {},
            loading: false,
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
        canOpen() {
            return this.items.length > 0
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
        toggle() {
            if(this.items.length == 0) return this.active = false
            this.active = !this.active
        },
        importAddons() {
            this.loading = true
            let items = this.items.filter(item => this.selected[item.publishedfileid] && item)
            let running = 0;
            let timer = setInterval(async() => {
                if(items.length == 0 && running == 0) {
                    this.$emit('refreshItems')
                    this.loading = false
                    return clearInterval(timer)
                }else if(running < 6) {
                    let item = items.shift();
                    running++
                    console.log('item', item)
                    await invoke("import_addon", { item, isWorkshop: false })
                    running--
                }
            }, 1000)
        },
        onSelectAll(state) {
            for(const item of this.items) {
                this.$set(this.selected, item.publishedfileid, state)
            }
        } 
    }
}
</script>