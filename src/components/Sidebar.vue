<template>
<nav class="panel is-primary sidebar">
    <p class="panel-heading">v{{ appData.app_version }}</p>
    <router-link class="panel-block is-block" :to="{ name: 'addons-manual' }">
        Managed Addons
        <span class="tag is-rounded is-pulled-right">{{ counts.addons }}</span>
    </router-link>
    <router-link class="panel-block is-block" :to="{ name: 'addons-workshop' }">
        Workshop Addons
        <span class="tag is-rounded is-pulled-right">{{ counts.workshop }}</span>
    </router-link>
    <div class="panel-block">
        <br>
    </div>
    <a class="panel-block" @click="emit('scan')">
        {{ props.scanActive ? 'Cancel Scan' : 'Start Scan' }}
    </a>
    <router-link class="panel-block" :to="{ name: 'settings' }">
        Settings
    </router-link>
    <router-link class="panel-block" :to="{ name: 'export'}">
        Export
    </router-link>
    <router-link class="panel-block" :to="{ name: 'logs'}">
        Logs
    </router-link>
    <!-- <a class="panel-block" @click="checkForUpdates">
        Check for Updates
    </a> -->

    <footer v-if="appData">
        v{{ appData.app_version }}
        <template v-if="appData.git_commit">
            git {{ appData.git_commit }}
        </template>
        {{ size[0] }}x{{ size[1] }}
    </footer>
</nav>
</template>

<style scoped>
.sidebar, .sidebar .panel-heading, .sidebar .panel-block {
    border-radius: 0;
}
.sidebar {
    height: 100%;
}
.sidebar .panel-block {
    padding-left: 1em;
}
.sidebar .panel-block.is-active {
    background-color: lightblue;
    font-weight: bold;
    color: black;
}
.sidebar {
    border: 1px solid lightgray;
}
.sidebar footer {
    position: absolute;
    bottom: 0;
    padding-left: 5px;
}
</style>

<script setup lang="ts">
import { AddonCounts, StaticAppData } from '../types/App.ts';
import { check } from '@tauri-apps/plugin-updater';
import { notify } from '@kyvg/vue3-notification';
import { onMounted, ref } from 'vue';

const props = defineProps<{
    scanActive: boolean,
    appData: StaticAppData,
    counts: AddonCounts
}>()
const emit = defineEmits(["scan"])

const size = ref([0, 0])

async function checkForUpdates() {
    const update = await check();
    if(!update) return notify({
        type: "info",
        title: "No update found"
    })
    console.info(
        `found update ${update.version} from ${update.date} with notes ${update.body}`
    );
    notify({
        type: "info",
        title: "Update Found",
        text: "Downloading update"
    })
    let downloaded = 0;
    let contentLength = 0;
    // alternatively we could also call update.download() and update.install() separately
    await update.downloadAndInstall((event) => {
        switch (event.event) {
            case 'Started':
                contentLength = event.data.contentLength ?? 0;
                console.log(`started downloading ${event.data.contentLength} bytes`);
                break;
            case 'Progress':
                downloaded += event.data.chunkLength;
                console.log(`downloaded ${downloaded} from ${contentLength}`);
                break;
            case 'Finished':
                console.log('download finished');
                break;
        }
    });

    console.log('update installed');
    notify({
        type: "info",
        title: "Update Complete",
        text: "Restart app to launch updated version"
    })
}

onMounted(() => {
    size.value = [window.outerWidth, window.outerHeight]
    addEventListener("resize", (_) => {
        size.value = [window.outerWidth, window.outerHeight]
    })
})
</script>