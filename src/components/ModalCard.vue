<template>
<div class="modal is-active">
  <div class="modal-background" @click="close"></div>
  <div class="modal-card">
    <header class="modal-card-head pb-4">
      <p class="modal-card-title">{{ props.title }}</p>
      <button class="delete" aria-label="close" @click="close"></button>
    </header>
    <section class="modal-card-body pt-4 pb-4">
       <slot />
    </section>
    <footer class="modal-card-foot">
        <slot name="footer">

        </slot>
    </footer>
  </div>
</div>
</template>

<script setup lang="ts">
const emit = defineEmits(["close"])

const props = defineProps<{
    title: string,
    active: boolean
}>()

function close() {
  emit("close")
}
</script>

<style scoped>
.modal .modal-card {
    width: 80%;
}
</style>