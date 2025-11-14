<template>
<div class="container mt-6 has-text-centered" style="background-color: rgba(255, 255, 255,1.067);">
    <h1 class="title is-1">Welcome to L4D2 Addon Manager</h1>
    <p class="subtitle is-3">First Time Setup</p>
    <br>
    <div class="box">
        <h4 class="title is-4">Path to L4D2 Addon Folder</h4>
        <div class="block">
            <input class="input" type="text" v-model="folderPath" />
            <p class="help has-text-left">Should end with something like <code>Left 4 Dead 2/left4dead2/addons</code></p>
        </div>
        <div class="buttons">
            <button class="button" @click="chooseFolder">Browse for left4dead2.exe</button>
            <button class="button is-primary" @click="setFolder" :disabled="canSave ? undefined : true">
                <Icon button inline icon="arrow-right" text-left="Begin" />
            </button>
        </div>
    </div>
</div>
</template>

<script setup lang="ts">
import { computed, onBeforeMount, ref } from 'vue';
import { getGameFolder, setGameFolder } from '../js/tauri.ts';
import { useRoute, useRouter } from 'vue-router';
import Icon from '../components/Icon.vue';

const route = useRoute()
const router = useRouter()

const folderPath = ref<string|null>(null)

async function chooseFolder() {
    folderPath.value = await getGameFolder()
    console.log(folderPath.value)
}

async function setFolder() {
    if(!canSave.value) return
    await setGameFolder(folderPath.value!)
    router.replace({ name: "addons-manual" })
}

const canSave = computed(() => {
    return folderPath.value && folderPath.value != ""
})

onBeforeMount(() => {
    folderPath.value = route.query.suggestion as string
})
</script>