<template>
<nav class="panel is-primary sidebar">
    <p class="panel-heading">L4D2 Addon Manager</p>
    <router-link class="panel-block" :to="{ name: 'addons-manual' }">
        Managed Addons
    </router-link>
    <router-link class="panel-block" :to="{ name: 'addons-workshop' }">
        Workshop Addons
    </router-link>
</nav>
</template>

<style scoped>
.sidebar, .sidebar .panel-heading {
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
</style>

<script setup lang="ts">

</script>