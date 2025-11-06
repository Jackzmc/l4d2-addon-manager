<template>
    <div class="columns is-gapless">
        <div class="column is-3" >
            <Sidebar @refresh-list="onRefreshRequest" />
        </div>
        <main class="column mt-3 section-component">
            <router-view v-slot="{ Component }">
                <component ref="view" :is="Component" />
            </router-view>
        </main>
    </div>
</template>

<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import { ref } from 'vue';
const view = ref()

// If a refresh is requested (from sidebar), tell child to refresh, if they can
function onRefreshRequest() {
    if(view.value.refresh) {
        view.value.refresh()
    }
}
</script>

<style>
.section-component {
  height: 720px !important;
  overflow: auto !important;
}
</style>