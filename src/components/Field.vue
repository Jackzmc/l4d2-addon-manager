<template>
<div :class="['field', {'is-horizontal': props.horizontal}]">
    <label v-if="label || slots.label" class="label"><slot name="label">{{ label }}</slot></label>
    <div :class="[{'control': !props.horizontal, 'has-icons-left': iconLeft, 'has-icons-right': iconRight}]">
      <slot />
      <span v-if="iconLeft" class="icon is-small is-left">
        <Icon button :icon="iconLeft" :class="iconClass" />
      </span>
      <span v-if="iconRight" class="icon is-small is-right">
        <Icon button :icon="iconRight" :class="iconClass" />
      </span>
    </div>
    <p v-if="success || slots.hint_success" class="help is-success"><slot name="hint_success">{{ success }}</slot></p>
    <p v-if="error || slots.hint_error" class="help is-danger"><slot name="hint_error">{{ error }}</slot></p>
    <p v-if="info || slots.hint_info" class="help is-info"><slot name="hint_info">{{ info }}</slot></p>
</div>
</template>

<script setup lang="ts">
import { Icon } from '@iconify/vue';
import { useSlots } from 'vue'
// import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
const slots = useSlots()
const props = defineProps<{
  horizontal?: boolean,
  label?: string,
  success?: string,
  error?: string,
  info?: string,
  iconLeft?: string,
  iconLeftSpin?: boolean,
  iconRight?: string
  iconRightSpin?: boolean,
  iconClass?: string
}>()
</script>
