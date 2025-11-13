<template>
  <span :class="(button ? ' ' : 'icon-text ') + (containerClass ?? '')">
    <span v-if="$slots.defaultLeft || textLeft" :class="textClass">
      <slot name="text-left">{{ textLeft }}</slot>
    </span>
    <span :class="'icon ' + (iconClass ?? '')">
      <Icon :icon="computedIcon" />
    </span>
    <span v-if="$slots.default || text" :class="textClass">
      <slot>{{ text }}</slot>
    </span>
  </span>
</template>

<script setup lang="ts">
const DEFAULT_ICON_SET = "iconoir" 
import { Icon } from '@iconify/vue'
import { computed } from 'vue';
const props = defineProps<{
  icon: string,
  text?: string,
  textLeft?: string
  containerClass?: string
  iconClass?: string,
  spin?: boolean,
  button?: boolean,
  rotate?: 90 | 180 | 270 | "90" | "180" | "270" | undefined,
  size?: "2xs" | "xs" | "sm" | "lg" | "xl" | "2xl" | "1x" | "2x" | "3x" | "4x" | "5x" | "6x" | "7x" | "8x" | "9x" | "10x" | undefined,
  counter?: number,
  layerClass?: string,
  textClass?: string
}>()

const computedIcon = computed(() => {
    if(props.icon.includes(":")) return props.icon
    return `${DEFAULT_ICON_SET}:${props.icon}`
})
</script>
