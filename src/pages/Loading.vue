<template>
<div class="container mt-6 has-text-centered">
    <div class="box has-background-link py-6">
        <h2 class="title is-2 has-text-white pt-6">L4D2 Addon Manager</h2>
        <p class="subtitle is-3 has-text-white py-2 pb-6">Loading{{ ".".repeat(ellipsCount) }}</p>
    </div>
</div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { init } from '../js/tauri.ts';
import { useRouter } from 'vue-router';

const emit = defineEmits(["init"])

const router = useRouter()

const ellipsCount = ref(3)

function cycleEllips() {
    const count = ellipsCount.value += 1
    ellipsCount.value = count % 3
}

onMounted(async() => {
    setInterval(cycleEllips, 750)

    const initData = await init()
    emit("init", initData)
    // send some init data as query, don't want to just keep track of it for no reason
    router.push({ name: initData.initial_route.name, query: {
        suggestion: initData.addon_folder_suggestion
    } })
})
</script>