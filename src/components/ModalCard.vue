<template>
<div class="modal is-active">
  <div class="modal-background" @click="close"></div>
  <div class="modal-card">
    <header class="modal-card-head">
      <p class="modal-card-title">{{ props.title }}</p>
      <button class="delete" aria-label="close" @click="close"></button>
    </header>
    <section class="modal-card-body">
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

  & .modal-card-head {
    padding-top: 20px;
    padding-bottom: 16px;
    /* border-radius: 0 !important; */
    border-bottom: 1px solid lightgray;
    font-weight: bold;
  }

  & .modal-card-body {
    padding-top: 12px;
    border-left: 4px solid white;
  }

  & .modal-card-foot {
    height: 80px;
    border: 1px solid lightgray;
    /* border-radius: 0 !important; */
  }
}
</style>