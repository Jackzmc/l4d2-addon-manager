<template>
<div class="container mt-6 has-text-centered" style="background-color: rgba(255, 255, 255,1.067);">
    <h1 class="title is-1">Welcome to L4D2 Addon Manager</h1>
    <p class="subtitle is-3">To begin, please select your <b>Left 4 Dead 2</b> folder</p>
    <br>
    <div class="box">
        <h4 class="title is-4">Selected Path</h4>
        <div class="block">
            <input class="input" type="text" v-model="folderPath" />
        </div>
        <div class="buttons">
            <button class="button" @click="chooseFolder">Browse</button>
            <button class="button is-primary" @click="setFolder" :disabled="canSave ? undefined : true">Continue</button>
        </div>
    </div>
</div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { getGameFolder, setGameFolder } from '../js/tauri.ts';
import { router } from '../router/index.ts';

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
</script>