<template>
<div>
    <div class="ml-5 my-3">
        <article class="message is-warning">
        <div class="message-body">
            <font-awesome-icon icon="exclamation-triangle" />
            These items do not include a publishedfileid, therefore cannot be managed by this tool. <br>
            Add their workshop id to the filename and refresh.
        </div>
        </article>
    </div>
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <th>Item Name</th>
                <th>File Size</th>
                <th>Last Updated</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="item in items" :key="item.publishedfileid" >
                <td @click="selected[item.publishedfileid] = !selected[item.publishedfileid]">{{item.title || item.publishedfileid}}.vpk</td>
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
    <hr>
</div>
</template>

<script>
import { formatBytes, formatDate } from '@/js/utils'

export default {
    props: ['items'],
    data() {
        return {
            active: false,
            selected: {},
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
        }
    },
    methods: {
        formatBytes, 
        formatDate,
        toggle() {
            if(this.items.length == 0) return this.active = false
            this.active = !this.active
        }
    }
}
</script>